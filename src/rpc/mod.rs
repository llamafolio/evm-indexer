use anyhow::Result;
use log::*;
use web3::{
    futures::StreamExt,
    transports::{Batch, Http, WebSocket},
    types::{Block, BlockId, Log, Transaction, TransactionReceipt, U64},
    Web3,
};

use crate::{
    config::Config,
    db::{
        models::{DatabaseBlock, DatabaseTx, DatabaseTxLogs, DatabaseTxReceipt},
        Database,
    },
    utils::{format_block, format_receipt},
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub batch: Web3<Batch<Http>>,
    pub wss: Web3<WebSocket>,
    pub chain: String,
}

impl Rpc {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Rpc");

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
                .map(|block| format_block(&block))
                .collect()),
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn get_txs_receipts(&self, txs: &Vec<Transaction>) -> Result<Vec<TransactionReceipt>> {
        for tx in txs.iter() {
            self.batch.eth().transaction_receipt(tx.hash);
        }
        let result = self.batch.transport().submit_batch().await;

        match result {
            Ok(result) => Ok(result
                .into_iter()
                .map(|receipt| format_receipt(&receipt))
                .collect()),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn subscribe_heads(&self, db: &Database) {
        let mut sub = self
            .wss
            .eth_subscribe()
            .subscribe_new_heads()
            .await
            .unwrap();

        info!("Initializing new heads listener with id {:?}", sub.id());

        loop {
            let new_block = sub.next().await;

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

                        let (db_blocks, db_txs, db_tx_receipts, db_tx_logs) =
                            self.get_blocks(range).await.unwrap();

                        db.store_blocks_and_txs(db_blocks, db_txs, db_tx_receipts, db_tx_logs)
                            .await;
                    }
                    Err(_) => {
                        continue;
                    }
                },
                None => {
                    continue;
                }
            }
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

        let web3_receipts = self.get_txs_receipts(&web3_txs).await.into_iter().flatten();

        let db_txs: Vec<DatabaseTx> = web3_txs
            .into_iter()
            .map(|tx| DatabaseTx::from_web3(&tx, self.chain.clone()))
            .collect();

        let (db_tx_receipts, web3_vec_tx_logs): (Vec<DatabaseTxReceipt>, Vec<Vec<Log>>) =
            web3_receipts
                .into_iter()
                .map(|tx_receipt| {
                    (
                        DatabaseTxReceipt::from_web3(tx_receipt.clone(), self.chain.clone()),
                        tx_receipt.logs,
                    )
                })
                .unzip();

        let db_tx_logs: Vec<DatabaseTxLogs> = web3_vec_tx_logs
            .into_iter()
            .flatten()
            .map(|log| DatabaseTxLogs::from_web3(log, self.chain.clone()))
            .collect();

        Ok((db_blocks, db_txs, db_tx_receipts, db_tx_logs))
    }
}
