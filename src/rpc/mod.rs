use std::str::FromStr;

use anyhow::Result;
use log::*;
use web3::{
    contract::{Contract, Options},
    ethabi::Address,
    futures::{future::join_all, StreamExt},
    transports::{Batch, Http, WebSocket},
    types::{Block, BlockId, Transaction, TransactionReceipt, U64},
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
    utils::{format_address, format_block, format_bool, format_receipt, ERC20_ABI},
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub single: Web3<Http>,
    pub batch: Web3<Batch<Http>>,
    pub wss: Option<Web3<WebSocket>>,
    pub chain: Chain,
    pub requests_batch: usize,
}

impl Rpc {
    pub async fn new(config: &Config, provider: &Provider) -> Result<Self> {
        let http = Http::new(&provider.http).unwrap();

        Ok(Self {
            wss: get_websocket(provider).await,
            single: Web3::new(http.clone()),
            batch: Web3::new(web3::transports::Batch::new(http)),
            chain: config.chain,
            requests_batch: config.batch_size.clone(),
        })
    }

    pub async fn get_last_block(&self) -> Result<i64> {
        let last_block = self.single.eth().block_number().await.unwrap().as_u64() as i64;

        Ok(last_block)
    }

    async fn get_block_batch(&self, range: &Vec<i64>) -> Result<Vec<Block<Transaction>>> {
        for block_height in range.iter() {
            let block_number = U64::from_str_radix(&block_height.to_string(), 10)
                .expect("Unable to parse block number");

            let block_id = <BlockId as From<U64>>::from(block_number);

            self.batch.eth().block_with_txs(block_id);
        }

        let blocks_res = self.batch.transport().submit_batch().await;

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

    async fn get_txs_receipts(&self, txs: &Vec<Transaction>) -> Result<Vec<TransactionReceipt>> {
        let mut receipts: Vec<TransactionReceipt> = Vec::new();

        if txs.len() == 0 {
            return Ok(receipts);
        }

        for tx in txs.iter() {
            self.batch.eth().transaction_receipt(tx.hash);
        }

        let receipts_res = self.batch.transport().submit_batch().await;

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

                                let from = block_number.as_u64() as i64 - self.chain.blocks_reorg;
                                let to = block_number.as_u64() as i64;

                                let range: Vec<i64> = (from..to).collect();

                                let (
                                    db_blocks,
                                    db_txs,
                                    db_tx_receipts,
                                    db_tx_logs,
                                    db_contract_creations,
                                    db_contract_interactions,
                                    db_token_transfers,
                                ) = self.get_blocks(range).await.unwrap();

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
        let block_chunks = range.chunks(self.requests_batch.clone());

        let mut blocks: Vec<Block<Transaction>> = Vec::new();

        for chunk in block_chunks {
            let mut block_chunk = self.get_block_batch(&chunk.to_vec()).await.unwrap();
            blocks.append(&mut block_chunk);
        }

        let (db_blocks, web3_vec_txs): (Vec<DatabaseBlock>, Vec<Vec<Transaction>>) = blocks
            .into_iter()
            .map(|block| {
                (
                    DatabaseBlock::from_web3(&block, self.chain.name.to_string()),
                    block.transactions,
                )
            })
            .unzip();

        let web3_txs: Vec<Transaction> = web3_vec_txs.into_iter().flatten().collect();

        let mut tx_receipts: Vec<TransactionReceipt> = Vec::new();

        let receipts_chunks = web3_txs.chunks(self.requests_batch.clone() / 2);

        for chunk in receipts_chunks {
            let mut receipts_chunk = self.get_txs_receipts(&chunk.to_vec()).await.unwrap();
            tx_receipts.append(&mut receipts_chunk);
        }

        let db_txs: Vec<DatabaseTx> = web3_txs
            .into_iter()
            .map(|tx| DatabaseTx::from_web3(&tx, self.chain.name.to_string()))
            .collect();

        let mut db_tx_receipts: Vec<DatabaseTxReceipt> = vec![];

        let mut db_tx_logs: Vec<DatabaseTxLogs> = vec![];

        let mut db_contract_creations: Vec<DatabaseContractCreation> = vec![];

        let mut db_contract_interactions: Vec<DatabaseContractInteraction> = vec![];

        let mut db_token_transfers: Vec<DatabaseTokenTransfers> = vec![];

        for tx_receipt in tx_receipts {
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

                                let db_log =
                                    DatabaseTxLogs::from_web3(log, self.chain.name.to_string());

                                db_tx_logs.push(db_log);
                            }
                        }
                    }
                }
            }
        }

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
                        format_address(address),
                        self.chain.name.to_string()
                    ),
                    address: format_address(address),
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

        let contract = Contract::from_json(self.batch.eth(), token, erc20_abi).unwrap();

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
