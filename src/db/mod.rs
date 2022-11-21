use anyhow::Result;
use log::*;
use web3::{
    types::{Block, Transaction},
    Error,
};

use crate::config::Config;

pub struct State {
    pub id: String,
    pub last_block: i64,
}

pub struct DatabaseBlock {
    pub height: i64,
    pub hash: String,
    pub txs: i64,
    pub timestamp: i64,
    pub size: i64,
    pub nonce: String,
}

pub struct DatabaseTx {
    pub hash: String,
    pub from_address: String,
    pub to_address: String,
    pub create_contract: bool,
    pub block: i64,
    pub tx_value: String,
    pub timestamp: i64,
    pub tx_index: i64,
    pub tx_type: i64,
    pub data_input: String,
    pub gas: String,
    pub gas_price: String,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub initial_block: usize,
}

impl Database {
    pub async fn new(config: Config, initial_block: usize) -> Result<Self> {
        info!("Initializing Database");

        Ok(Self { initial_block })
    }

    pub async fn last_synced_block(&self) -> Result<i64> {
        Ok(0)
    }

    pub async fn store_block_batch(
        &self,
        blocks: Vec<Result<serde_json::Value, Error>>,
        update_sync_state: bool,
    ) {
    }

    pub async fn store_txs_batch(&self, blocks: Vec<Result<serde_json::Value, Error>>) {}

    pub async fn update_sync_state(&self, last_block: i64) {}

    pub async fn store_block(&self, block: Block<Transaction>) {}

    pub async fn store_block_txs(&self, block: Block<Transaction>) {}
}
