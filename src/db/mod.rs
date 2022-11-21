mod models;
mod schema;

use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel::PgConnection;
use log::*;
use web3::Error;

use crate::config::Config;
use crate::utils::format_block;

use self::models::DatabaseBlock;
use self::schema::blocks;

pub struct State {
    pub id: String,
    pub last_block: i64,
}

pub struct Database {
    pub db_url: String,
    pub initial_block: usize,
}

impl Database {
    pub async fn new(config: Config, initial_block: usize) -> Result<Self> {
        info!("Initializing Database");

        Ok(Self {
            initial_block,
            db_url: config.db_url,
        })
    }

    fn establish_connection(&self) -> PgConnection {
        let connection =
            PgConnection::establish(&self.db_url).expect("Unable to connect to the database");

        return connection;
    }

    pub async fn last_synced_block(&self) -> Result<i64> {
        Ok(0)
    }

    pub async fn store_blocks(
        &self,
        blocks: Vec<Result<serde_json::Value, Error>>,
        update_sync_state: bool,
    ) {
        let db_blocks: Vec<DatabaseBlock> = blocks
            .iter()
            .map(|block| DatabaseBlock::from_web3_block(format_block(block)))
            .collect();

        let mut connection = self.establish_connection();

        insert_into(blocks::table)
            .values(&db_blocks)
            .on_conflict(blocks::number)
            .do_update()
            .set(blocks::all_columns.eq(excluded(blocks::number)))
            .execute(&mut connection)
            .expect("Unable to store blocks in the database");

        info!("Inserted {} blocks to the database", db_blocks.len());
    }

    pub async fn store_txs(&self, blocks: Vec<Result<serde_json::Value, Error>>) {}

    pub async fn update_sync_state(&self, last_block: i64) {}
}
