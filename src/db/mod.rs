pub mod models;
mod schema;

use anyhow::Result;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel_migrations::*;
use log::*;
use web3::futures::future::join_all;
use web3::futures::future::BoxFuture;

use crate::config::Config;

use self::models::DatabaseBlock;

use self::models::DatabaseState;
use self::models::DatabaseTx;
use self::models::DatabaseTxLogs;
use self::models::DatabaseTxReceipt;
use self::schema::state::dsl::state;
use self::schema::state::id;
use self::schema::state::last_block;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

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

        let mut connection =
            PgConnection::establish(&config.db_url).expect("Unable to connect to the database");

        connection.run_pending_migrations(MIGRATIONS).unwrap();

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
        let mut connection = self.establish_connection();

        let state_data: Result<DatabaseState, diesel::result::Error> = state
            .filter(id.eq(String::from("state")))
            .first(&mut connection);

        let last_block_number: i64 = match state_data {
            Ok(data) => data.last_block,
            Err(_) => self.initial_block as i64,
        };

        Ok(last_block_number)
    }

    pub async fn store_blocks_and_txs(
        &self,
        blocks: Vec<DatabaseBlock>,
        txs: Vec<DatabaseTx>,
        receipts: Vec<DatabaseTxReceipt>,
        logs: Vec<DatabaseTxLogs>,
    ) {
        let mut stores: Vec<BoxFuture<_>> = vec![];

        if blocks.len() > 0 {
            stores.push(Box::pin(self.store_blocks(&blocks)))
        }

        if txs.len() > 0 {
            stores.push(Box::pin(self.store_txs(&txs)));
        }

        if receipts.len() > 0 {
            stores.push(Box::pin(self.store_tx_receipts(&receipts)));
        }

        if logs.len() > 0 {
            stores.push(Box::pin(self.store_tx_logs(&logs)));
        }

        join_all(stores).await;

        /* self.update_sync_state(blocks.last().unwrap().number)
        .await
        .unwrap(); */
    }

    async fn store_blocks(&self, blocks: &Vec<DatabaseBlock>) -> Result<()> {
        let mut connection = self.establish_connection();

        diesel::insert_into(schema::blocks::dsl::blocks)
            .values(blocks)
            .on_conflict_do_nothing()
            .execute(&mut connection)
            .expect("Unable to store blocks in the database");

        info!("Inserted {} blocks to the database", blocks.len());

        Ok(())
    }

    async fn store_txs(&self, txs: &Vec<DatabaseTx>) -> Result<()> {
        let mut connection = self.establish_connection();

        diesel::insert_into(schema::txs::dsl::txs)
            .values(txs)
            .on_conflict_do_nothing()
            .execute(&mut connection)
            .expect("Unable to store txs in the database");

        info!("Inserted {} txs to the database", txs.len());

        Ok(())
    }

    async fn store_tx_receipts(&self, tx_receipts: &Vec<DatabaseTxReceipt>) -> Result<()> {
        let mut connection = self.establish_connection();

        diesel::insert_into(schema::txs_receipts::dsl::txs_receipts)
            .values(tx_receipts)
            .on_conflict_do_nothing()
            .execute(&mut connection)
            .expect("Unable to store tx_receipts in the database");

        info!("Inserted {} tx_receipts to the database", tx_receipts.len());

        Ok(())
    }

    async fn store_tx_logs(&self, logs: &Vec<DatabaseTxLogs>) -> Result<()> {
        let mut connection = self.establish_connection();

        diesel::insert_into(schema::logs::dsl::logs)
            .values(logs)
            .on_conflict_do_nothing()
            .execute(&mut connection)
            .expect("Unable to store logs in the database");

        info!("Inserted {} logs to the database", logs.len());

        Ok(())
    }

    pub async fn update_sync_state(&self, last_block_number: i64) -> Result<()> {
        let mut connection = self.establish_connection();

        let state_data = DatabaseState {
            id: String::from("state"),
            last_block: last_block_number,
        };

        diesel::insert_into(state)
            .values(state_data)
            .on_conflict(id)
            .do_update()
            .set(last_block.eq(last_block_number))
            .execute(&mut connection)
            .expect("Unable to update sync state last blocks");

        info!("Updated last sync state to block {}", last_block_number);

        Ok(())
    }
}
