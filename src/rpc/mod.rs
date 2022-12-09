mod client;

use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use jsonrpsee::core::{client::ClientT, params::BatchRequestBuilder, rpc_params};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use log::*;

use reth_primitives::rpc::{Block, Transaction, TransactionReceipt};

use web3::{
    contract::{Contract, Options},
    ethabi::Address,
    futures::{future::join_all, StreamExt},
    transports::{Http, WebSocket},
    Web3,
};

use crate::{
    chains::{Chain, Provider},
    config::Config,
    db::{
        models::{
            token_transfers_from_logs, DatabaseBlock, DatabaseContractCreation,
            DatabaseContractInteraction, DatabaseToken, DatabaseTokenTransfers, DatabaseTx,
            DatabaseTxLogs, DatabaseTxReceipt,
        },
        Database,
    },
    utils::{
        format_address, format_block, format_bool, format_bytes, format_hash, format_receipt,
        ERC20_ABI,
    },
};

use self::client::EthApiClient;

#[derive(Debug, Clone)]
pub struct Rpc {
    pub web3: Web3<Http>,
    pub http_client: HttpClient,
    pub wss: Option<Web3<WebSocket>>,
    pub chain: Chain,
    pub requests_batch: usize,
}

impl Rpc {
    pub async fn new(config: &Config, provider: &Provider) -> Result<Self> {
        let http = Http::new(&provider.http).unwrap();

        let client = HttpClientBuilder::default()
            .build(&provider.http.clone())
            .unwrap();

        Ok(Self {
            wss: get_websocket(provider).await,
            http_client: client,
            chain: config.chain,
            requests_batch: config.batch_size.clone(),
            web3: Web3::new(http.clone()),
        })
    }

    pub async fn get_last_block(&self) -> Result<i64> {
        let eth_client = EthApiClient::block_number(&self.http_client)
            .await
            .unwrap()
            .as_u64() as i64;

        Ok(eth_client)
    }

    async fn get_block_batch(&self, range: &Vec<i64>) -> Result<Vec<Block<Transaction>>> {
        let mut batch = BatchRequestBuilder::new();

        for block_height in range.iter() {
            batch
                .insert(
                    "eth_getBlockByNumber",
                    rpc_params![format!("0x{:x}", block_height), true],
                )
                .unwrap();
        }

        let blocks_res = self.http_client.batch_request(batch).await;

        match blocks_res {
            Ok(result) => {
                let mut blocks: Vec<Block<Transaction>> = Vec::new();

                for block in result.into_iter() {
                    match block {
                        Ok(block) => match format_block(block) {
                            Ok(block_formated) => blocks.push(block_formated),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    }
                }

                Ok(blocks)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_txs_receipts(&self, txs: &Vec<String>) -> Result<Vec<TransactionReceipt>> {
        let mut batch = BatchRequestBuilder::new();

        let mut receipts: Vec<TransactionReceipt> = Vec::new();

        if txs.len() == 0 {
            return Ok(receipts);
        }

        for tx in txs.iter() {
            batch
                .insert("eth_getTransactionReceipt", rpc_params![tx])
                .unwrap();
        }

        let receipts_res = self.http_client.batch_request(batch).await;

        match receipts_res {
            Ok(result) => {
                for receipt in result.into_iter() {
                    match receipt {
                        Ok(tx_receipt) => match format_receipt(tx_receipt) {
                            Ok(receipt_formatted) => receipts.push(receipt_formatted),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    }
                }

                Ok(receipts)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn subscribe_heads(&self, db: &Database) {
        match self.wss.clone() {
            Some(wss) => {
                let mut sub = wss.eth_subscribe().subscribe_new_heads().await.unwrap();

                info!("Initializing new blocks listener");

                loop {
                    let new_block = sub.next().await;
                    match new_block {
                        Some(block_header) => match block_header {
                            Ok(block_header) => {
                                let block_number = block_header.number.unwrap();
                                info!(
                                    "Received new block header with height {:?} for chain {}",
                                    block_header.number.unwrap(),
                                    self.chain.name
                                );

                                let (
                                    db_blocks,
                                    db_txs,
                                    db_tx_receipts,
                                    db_tx_logs,
                                    db_contract_creations,
                                    db_contract_interactions,
                                    db_token_transfers,
                                ) = self
                                    .get_blocks([block_number.as_u64() as i64].to_vec())
                                    .await
                                    .unwrap();

                                db.store_blocks_and_txs(
                                    db_blocks,
                                    db_txs,
                                    db_tx_receipts,
                                    db_tx_logs,
                                    db_contract_creations,
                                    db_contract_interactions,
                                    db_token_transfers,
                                )
                                .await;
                            }
                            Err(_) => {
                                return;
                            }
                        },
                        None => {
                            return;
                        }
                    }
                }
            }
            None => return,
        }
    }
    pub async fn get_blocks(
        &self,
        range: Vec<i64>,
    ) -> Result<(
        Vec<DatabaseBlock>,
        Vec<DatabaseTx>,
        Vec<DatabaseTxReceipt>,
        Vec<DatabaseTxLogs>,
        Vec<DatabaseContractCreation>,
        Vec<DatabaseContractInteraction>,
        Vec<DatabaseTokenTransfers>,
    )> {
        let blocks = self.get_block_batch(&range).await.unwrap();

        let (db_blocks, web3_vec_txs): (Vec<DatabaseBlock>, Vec<Vec<Transaction>>) = blocks
            .into_iter()
            .map(|block| {
                (
                    DatabaseBlock::from_web3(&block, self.chain.name.to_string()),
                    block.transactions,
                )
            })
            .unzip();

        let blocks_timestamps = db_blocks
            .clone()
            .into_iter()
            .map(|block| (block.hash, block.timestamp));

        let block_timestamps_hashmap: HashMap<String, String> =
            HashMap::from_iter(blocks_timestamps);

        let web3_txs: Vec<Transaction> = web3_vec_txs.into_iter().flatten().collect();

        let mut tx_receipts: Vec<TransactionReceipt> = Vec::new();

        let receipts_chunks = web3_txs.chunks(self.requests_batch.clone() / 2);

        for chunk in receipts_chunks {
            let tx_hashes: Vec<String> = chunk.into_iter().map(|tx| format_hash(tx.hash)).collect();
            let mut receipts_chunk = self.get_txs_receipts(&tx_hashes).await.unwrap();
            tx_receipts.append(&mut receipts_chunk);
        }

        let db_txs: Vec<DatabaseTx> = web3_txs
            .into_iter()
            .map(|tx| {
                let block_hash = format_hash(tx.block_hash.unwrap());
                DatabaseTx::from_web3(
                    &tx,
                    self.chain.name.to_string(),
                    block_timestamps_hashmap.get(&block_hash),
                )
            })
            .collect();

        let (
            db_tx_receipts,
            db_tx_logs,
            db_contract_creations,
            db_contract_interactions,
            db_token_transfers,
        ) = self.get_metadata_from_receipts(tx_receipts).await.unwrap();

        Ok((
            db_blocks,
            db_txs,
            db_tx_receipts,
            db_tx_logs,
            db_contract_creations,
            db_contract_interactions,
            db_token_transfers,
        ))
    }

    pub async fn get_metadata_from_receipts(
        &self,
        receipts: Vec<TransactionReceipt>,
    ) -> Result<(
        Vec<DatabaseTxReceipt>,
        Vec<DatabaseTxLogs>,
        Vec<DatabaseContractCreation>,
        Vec<DatabaseContractInteraction>,
        Vec<DatabaseTokenTransfers>,
    )> {
        let mut db_tx_receipts: Vec<DatabaseTxReceipt> = vec![];

        let mut db_tx_logs: Vec<DatabaseTxLogs> = vec![];

        let mut db_contract_creations: Vec<DatabaseContractCreation> = vec![];

        let mut db_contract_interactions: Vec<DatabaseContractInteraction> = vec![];

        let mut db_token_transfers: Vec<DatabaseTokenTransfers> = vec![];

        for tx_receipt in receipts {
            let db_tx_receipt =
                DatabaseTxReceipt::from_web3(tx_receipt.clone(), self.chain.name.to_string());

            db_tx_receipts.push(db_tx_receipt);

            let success: bool = match tx_receipt.status {
                None => false,
                Some(success) => format_bool(success),
            };

            if success {
                let logs = tx_receipt.logs.clone();

                match tx_receipt.contract_address {
                    Some(contract) => {
                        db_contract_creations.push(DatabaseContractCreation::from_receipt(
                            &tx_receipt,
                            self.chain.name.to_string(),
                            format_address(contract),
                        ))
                    }
                    None => {
                        if logs.len() > 0 {
                            let db_contract_interaction = DatabaseContractInteraction::from_receipt(
                                &tx_receipt,
                                self.chain.name.to_string(),
                            );

                            db_contract_interactions.push(db_contract_interaction);

                            // Check for token transfers
                            for log in logs {
                                match token_transfers_from_logs(
                                    &log,
                                    &tx_receipt,
                                    self.chain.name.to_string(),
                                ) {
                                    Ok(token_transfer) => db_token_transfers.push(token_transfer),
                                    Err(_) => continue,
                                };

                                let db_log = {
                                    let log = log;
                                    let chain = self.chain.name.to_string();
                                    let transaction_log_index: i64 = match log.transaction_log_index
                                    {
                                        None => 0,
                                        Some(transaction_log_index) => {
                                            transaction_log_index.as_u64() as i64
                                        }
                                    };

                                    let log_type: String = match log.log_type {
                                        None => String::from(""),
                                        Some(log_type) => log_type,
                                    };

                                    let hash = format_hash(log.transaction_hash.unwrap());
                                    DatabaseTxLogs {
                                        hash_with_index: format!(
                                            "{}-{}",
                                            hash,
                                            log.log_index.unwrap().as_u64()
                                        ),
                                        hash: format_hash(log.transaction_hash.unwrap()),
                                        address: format_address(log.address),
                                        data: format_bytes(&log.data),
                                        log_index: log.log_index.unwrap().as_u64() as i64,
                                        transaction_log_index,
                                        log_type,
                                        topics: log
                                            .topics
                                            .into_iter()
                                            .map(|topic| format_hash(topic))
                                            .collect(),
                                        chain,
                                    }
                                };

                                db_tx_logs.push(db_log);
                            }
                        }
                    }
                }
            }
        }

        Ok((
            db_tx_receipts,
            db_tx_logs,
            db_contract_creations,
            db_contract_interactions,
            db_token_transfers,
        ))
    }

    pub async fn get_tokens_metadata(&self, tokens: Vec<String>) -> Result<Vec<DatabaseToken>> {
        let mut tokens_metadata: Vec<DatabaseToken> = Vec::new();

        let tokens: Vec<Address> = tokens
            .into_iter()
            .map(|token| Address::from_str(&token).unwrap())
            .collect();

        let mut tokens_req = vec![];

        for token in tokens {
            tokens_req.push(self.get_erc20_details(token));
        }

        let tokens_data = join_all(tokens_req).await;

        for token_data in tokens_data {
            match token_data {
                Ok((name, symbol, decimals, address)) => tokens_metadata.push(DatabaseToken {
                    address_with_chain: format!(
                        "{}-{}",
                        format!("{:?}", address),
                        self.chain.name.to_string()
                    ),
                    address: format!("{:?}", address),
                    chain: self.chain.name.to_string(),
                    // Name and Symbol are reformatted to prevent non utf8 characters
                    name: format!("{}", name),
                    symbol: format!("{}", symbol),
                    decimals,
                }),
                Err(_) => continue,
            }
        }

        Ok(tokens_metadata)
    }

    pub async fn get_erc20_details(
        &self,
        token: Address,
    ) -> Result<(String, String, i64, Address), anyhow::Error> {
        let erc20_abi = ERC20_ABI;

        let contract = Contract::from_json(self.web3.eth(), token, erc20_abi).unwrap();

        let name: String = match contract
            .query("name", (), None, Options::default(), None)
            .await
        {
            Ok(result) => result,
            Err(_) => return Err(anyhow::Error::msg("Invalid token name")),
        };

        let decimals: i64 = match contract
            .query("decimals", (), None, Options::default(), None)
            .await
        {
            Ok(result) => result,
            Err(_) => return Err(anyhow::Error::msg("Invalid token decimals")),
        };

        let symbol: String = match contract
            .query("symbol", (), None, Options::default(), None)
            .await
        {
            Ok(result) => result,
            Err(_) => return Err(anyhow::Error::msg("Invalid token symbol")),
        };

        Ok((name, symbol, decimals, token))
    }
}

async fn get_websocket(provider: &Provider) -> Option<Web3<WebSocket>> {
    if !provider.wss_access {
        None
    } else {
        let wss = WebSocket::new(&provider.wss).await.unwrap();
        Some(Web3::new(wss))
    }
}
