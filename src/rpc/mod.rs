use std::str::FromStr;

use anyhow::Result;
use log::*;
use web3::{
    contract::{Contract, Options},
    ethabi::Address,
    futures::{future::join_all, StreamExt},
    transports::{Batch, Http, WebSocket},
    types::{Block, BlockId, BlockNumber, Transaction, TransactionReceipt, U64},
    Web3,
};

use crate::{
    chains::Chain,
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
        format_address, format_block, format_bool, format_receipt, format_receipts, ERC20_ABI,
    },
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub batch: Web3<Batch<Http>>,
    pub wss: Web3<WebSocket>,
    pub chain: Chain,
}

impl Rpc {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing RPC");

        let http = Http::new(&config.rpc_http_url).unwrap();
        let ws = WebSocket::new(&config.rpc_ws_url).await.unwrap();

        Ok(Self {
            wss: Web3::new(ws),
            batch: Web3::new(web3::transports::Batch::new(http)),
            chain: config.chain,
        })
    }

    pub async fn get_last_block(&self) -> Result<i64> {
        let last_block = self
            .wss
            .eth()
            .block_number()
            .await
            .expect("Unable to fetch last block")
            .as_u64();

        Ok(last_block as i64)
    }

    async fn get_block_batch(&self, range: Vec<i64>) -> Result<Vec<Block<Transaction>>> {
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
        let chunks = txs.chunks(200);

        let mut responses = Vec::new();

        for chunk in chunks {
            for tx in chunk.iter() {
                self.batch.eth().transaction_receipt(tx.hash);
            }

            let receipt_res = self.batch.transport().submit_batch().await;
            match receipt_res {
                Ok(mut result) => responses.append(&mut result),
                Err(_) => continue,
            };
        }

        let mut receipts_vec: Vec<TransactionReceipt> = Vec::new();

        for receipts in responses.into_iter() {
            match receipts {
                Ok(receipts) => match format_receipt(receipts) {
                    Ok(receipts_formatted) => receipts_vec.push(receipts_formatted),
                    Err(_) => continue,
                },
                Err(_) => continue,
            }
        }

        Ok(receipts_vec)
    }

    async fn get_block_receipts(
        &self,
        blocks: &Vec<Block<Transaction>>,
    ) -> Result<Vec<TransactionReceipt>> {
        let chunks = blocks.chunks(200);

        let mut responses = Vec::new();

        for chunk in chunks {
            for block in chunk.iter() {
                self.batch
                    .eth()
                    .block_receipts(BlockNumber::Number(block.number.unwrap()));
            }

            let receipt_res = self.batch.transport().submit_batch().await;
            match receipt_res {
                Ok(mut result) => responses.append(&mut result),
                Err(_) => continue,
            };
        }

        let mut receipts_vec: Vec<TransactionReceipt> = Vec::new();

        for receipts in responses.into_iter() {
            match receipts {
                Ok(receipts) => receipts_vec.append(&mut format_receipts(receipts)),
                Err(_) => continue,
            }
        }

        Ok(receipts_vec)
    }

    pub async fn subscribe_heads(&self, db: &Database) {
        let mut sub = self
            .wss
            .eth_subscribe()
            .subscribe_new_heads()
            .await
            .unwrap();

        info!("Initializing new blocks listener");

        loop {
            let new_block = sub.next().await;

            let rpc = self.clone();
            let spawn_db = db.clone();
            let chain = self.chain.clone();

            tokio::spawn(async move {
                match new_block {
                    Some(block_header) => match block_header {
                        Ok(block_header) => {
                            let block_number = block_header.number.unwrap();
                            info!(
                                "Received new block header with height {:?} for chain {}",
                                block_header.number.unwrap(),
                                chain.name
                            );

                            let from = block_number.as_u64() as i64 - 5;
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
                            ) = rpc.get_blocks(range).await.unwrap();

                            spawn_db
                                .store_blocks_and_txs(
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
            });
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
        let blocks = self.get_block_batch(range).await.unwrap();

        let mut web3_receipts: Vec<TransactionReceipt> = Vec::new();

        if self.chain.name.to_string() == "mainnet" {
            let mut receipts = self.get_block_receipts(&blocks).await.unwrap();
            web3_receipts.append(&mut receipts);
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

        if self.chain.name.to_string() != "mainnet" {
            let mut receipts = self.get_txs_receipts(&web3_txs).await.unwrap();
            web3_receipts.append(&mut receipts);
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

        for tx_receipt in web3_receipts {
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
                        "{:?}-{:?}",
                        format_address(address),
                        self.chain.name.to_string()
                    ),
                    address: format_address(address),
                    chain: self.chain.name.to_string(),
                    name,
                    decimals,
                    symbol,
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
