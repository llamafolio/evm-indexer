use anyhow::Result;
use log::*;
use web3::{
    futures::StreamExt,
    transports::{Batch, Http, WebSocket},
    types::{Block, BlockId, Transaction, TransactionReceipt, U64},
    Error, Web3,
};

use crate::{
    config::Config,
    db::Database,
    utils::{format_block, format_receipt},
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub batch: Web3<Batch<Http>>,
    pub wss: Web3<WebSocket>,
}

impl Rpc {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Rpc");

        let http = Http::new(&config.rpc_http_url).unwrap();
        let ws = WebSocket::new(&config.rpc_ws_url).await.unwrap();

        Ok(Self {
            wss: Web3::new(ws),
            batch: Web3::new(web3::transports::Batch::new(http)),
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

    pub async fn get_block_batch(&self, range: Vec<i64>) -> Result<Vec<Block<Transaction>>> {
        for block_height in range.iter() {
            let block_number = U64::from_str_radix(&block_height.to_string(), 10)
                .expect("Unable to parse block number");

            let block_id = <BlockId as From<U64>>::from(block_number);

            self.batch.eth().block_with_txs(block_id);
        }

        let result = self.batch.transport().submit_batch().await;

        match result {
            Ok(result) => Ok(result
                .into_iter()
                .map(|block| format_block(&block))
                .collect()),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_txs_receipts(&self, txs: Vec<Transaction>) -> Result<Vec<TransactionReceipt>> {
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

                        let web3_blocks = self.get_block_batch(range).await.unwrap();

                        //db.store_blocks(&web3_blocks).await;
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
}
