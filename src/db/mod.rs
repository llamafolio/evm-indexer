mod models;
mod schema;

use anyhow::Result;
use diesel::prelude::*;
use diesel::PgConnection;
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

pub struct Database {
    pub initial_block: usize,
    pub connection: PgConnection,
}

impl Database {
    pub async fn new(config: Config, initial_block: usize) -> Result<Self> {
        info!("Initializing Database");
        let connection =
            PgConnection::establish(&config.db_url).expect("Unable to connect to the database");

        Ok(Self {
            connection,
            initial_block,
        })
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
