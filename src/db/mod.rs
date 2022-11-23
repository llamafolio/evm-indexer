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

use self::models::DatabaseTx;
use self::models::DatabaseTxLogs;
use self::models::DatabaseTxReceipt;
use self::schema::blocks;
use self::schema::blocks::table as blocks_table;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub struct State {
    pub id: String,
    pub last_block: i64,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub db_url: String,
}

impl Database {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Database");

        let mut connection =
            PgConnection::establish(&config.db_url).expect("Unable to connect to the database");

        connection.run_pending_migrations(MIGRATIONS).unwrap();

        Ok(Self {
            db_url: config.db_url,
        })
    }

    fn establish_connection(&self) -> PgConnection {
        let connection =
            PgConnection::establish(&self.db_url).expect("Unable to connect to the database");

        return connection;
    }

    pub async fn get_last_block(&self) -> Result<i64> {
        let mut connection = self.establish_connection();

        let last_block: Result<DatabaseBlock, diesel::result::Error> = blocks_table
            .order_by(blocks::number.desc())
            .first(&mut connection);

        let last_block_number: i64 = match last_block {
            Ok(data) => data.number,
            Err(_) => 0,
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

        for chunk in blocks.chunks(500) {
            diesel::insert_into(schema::blocks::dsl::blocks)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store blocks in the database");
        }

        info!("Inserted {} blocks to the database", blocks.len());

        Ok(())
    }

    async fn store_txs(&self, txs: &Vec<DatabaseTx>) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in txs.chunks(500) {
            diesel::insert_into(schema::txs::dsl::txs)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store txs in the database");
        }

        info!("Inserted {} txs to the database", txs.len());

        Ok(())
    }

    async fn store_tx_receipts(&self, tx_receipts: &Vec<DatabaseTxReceipt>) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in tx_receipts.chunks(500) {
            diesel::insert_into(schema::txs_receipts::dsl::txs_receipts)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store tx_receipts in the database");
        }

        info!("Inserted {} tx_receipts to the database", tx_receipts.len());

        Ok(())
    }

    async fn store_tx_logs(&self, logs: &Vec<DatabaseTxLogs>) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in logs {
            diesel::insert_into(schema::logs::dsl::logs)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store logs in the database");
        }

        info!("Inserted {} logs to the database", logs.len());

        Ok(())
    }
}
