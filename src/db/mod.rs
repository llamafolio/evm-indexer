mod models;
mod schema;

use anyhow::Result;
use diesel::prelude::*;
use diesel::PgConnection;
use log::*;
use web3::types::Block;
use web3::types::Transaction;
use web3::Error;

use crate::config::Config;
use crate::utils::format_block;

use self::models::DatabaseBlock;

use self::models::DatabaseTx;

pub struct State {
    pub id: String,
    pub last_block: i64,
}

#[derive(Debug, Clone)]
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
        let connection = self.establish_connection();

        /*
        match state_data {
            Ok(data) => {
                if data.len() <= 0 {
                    Ok(self.initial_block as i64)
                } else {
                    Ok(self.initial_block as i64)
                }
            }
            Err(_) => Ok(self.initial_block as i64),
        } */
        Ok(0)
    }

    pub async fn store_blocks(
        &self,
        res_blocks: Vec<Result<serde_json::Value, Error>>,
        update_sync_state: bool,
    ) {
        let web3_blocks: Vec<Block<Transaction>> =
            res_blocks.iter().map(|block| format_block(block)).collect();

        let db_blocks: Vec<DatabaseBlock> = web3_blocks
            .iter()
            .map(|block| DatabaseBlock::from_web3_block(block))
            .collect();

        let mut connection = self.establish_connection();

        diesel::insert_into(schema::blocks::dsl::blocks)
            .values(&db_blocks)
            .on_conflict_do_nothing()
            .execute(&mut connection)
            .expect("Unable to store blocks in the database");

        info!("Inserted {} blocks to the database", db_blocks.len());

        if update_sync_state {
            let last_block = db_blocks.last().unwrap();
            self.update_sync_state(last_block.number).await;
        }

        self.store_txs(web3_blocks, &mut connection).await;
    }

    async fn store_txs(&self, blocks: Vec<Block<Transaction>>, conn: &mut PgConnection) {
        let mut txs: Vec<DatabaseTx> = Vec::new();

        for block in blocks {
            for tx in block.transactions {
                let db_tx = DatabaseTx::from_web3_tx(tx);
                txs.push(db_tx);
            }
        }

        if txs.len() > 0 {
            diesel::insert_into(schema::txs::dsl::txs)
                .values(&txs)
                .on_conflict_do_nothing()
                .execute(conn)
                .expect("Unable to store txs in the database");

            info!("Inserted {} txs to the database", txs.len());
        }
    }

    pub async fn update_sync_state(&self, last_block: i64) {}
}
