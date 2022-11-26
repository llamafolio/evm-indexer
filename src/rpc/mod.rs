use anyhow::Result;
use log::*;
use web3::{
    futures::StreamExt,
    transports::{Batch, Http, WebSocket},
    types::{Block, BlockId, Transaction, TransactionReceipt, U64},
    Web3,
};

use crate::{
    config::Config,
    db::{
        models::{
            token_transfers_from_logs, DatabaseBlock, DatabaseContractCreation,
            DatabaseContractInteraction, DatabaseTokenTransfers, DatabaseTx, DatabaseTxLogs,
            DatabaseTxReceipt,
        },
        Database,
    },
    utils::{format_address, format_block, format_bool, format_receipt},
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub batch: Web3<Batch<Http>>,
    pub wss: Web3<WebSocket>,
    pub chain: String,
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
            Ok(result) => Ok(result
                .into_iter()
                .map(Result::unwrap)
                .map(|block| format_block(block))
                .collect()),
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

        let receipts = responses
            .into_iter()
            .map(Result::unwrap)
            .filter(|raw_receipt| !raw_receipt.is_null())
            .map(|raw_receipt| format_receipt(raw_receipt))
            .collect();

        Ok(receipts)
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

            tokio::spawn(async move {
                match new_block {
                    Some(block_header) => match block_header {
                        Ok(block_header) => {
                            let block_number = block_header.number.unwrap();
                            info!(
                                "Received new block header with height {:?}",
                                block_header.number.unwrap()
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

        let (db_blocks, web3_vec_txs): (Vec<DatabaseBlock>, Vec<Vec<Transaction>>) = blocks
            .into_iter()
            .map(|block| {
                (
                    DatabaseBlock::from_web3(&block, self.chain.clone()),
                    block.transactions,
                )
            })
            .unzip();

        let web3_txs: Vec<Transaction> = web3_vec_txs.into_iter().flatten().collect();

        let web3_receipts = self.get_txs_receipts(&web3_txs).await.unwrap();

        let db_txs: Vec<DatabaseTx> = web3_txs
            .into_iter()
            .map(|tx| DatabaseTx::from_web3(&tx, self.chain.clone()))
            .collect();

        let mut db_tx_receipts: Vec<DatabaseTxReceipt> = vec![];

        let mut db_tx_logs: Vec<DatabaseTxLogs> = vec![];

        let mut db_contract_creations: Vec<DatabaseContractCreation> = vec![];

        let mut db_contract_interactions: Vec<DatabaseContractInteraction> = vec![];

        let mut db_token_transfers: Vec<DatabaseTokenTransfers> = vec![];

        for tx_receipt in web3_receipts {
            let db_tx_receipt =
                DatabaseTxReceipt::from_web3(tx_receipt.clone(), self.chain.clone());

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
                            self.chain.clone(),
                            format_address(contract),
                        ))
                    }
                    None => {
                        if logs.len() > 0 {
                            let db_contract_interaction = DatabaseContractInteraction::from_receipt(
                                &tx_receipt,
                                self.chain.clone(),
                            );

                            db_contract_interactions.push(db_contract_interaction);

                            // Check for token transfers
                            for log in logs {
                                match token_transfers_from_logs(
                                    &log,
                                    &tx_receipt,
                                    self.chain.clone(),
                                ) {
                                    Ok(token_transfer) => db_token_transfers.push(token_transfer),
                                    Err(_) => continue,
                                };

                                let db_log = DatabaseTxLogs::from_web3(log, self.chain.clone());

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
}
