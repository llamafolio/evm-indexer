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

use self::models::DatabaseContractCreation;
use self::models::DatabaseContractInteraction;
use self::models::DatabaseTokenTransfers;
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
    pub chain: String,
}

impl Database {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Database");

        let mut connection =
            PgConnection::establish(&config.db_url).expect("Unable to connect to the database");

        connection.run_pending_migrations(MIGRATIONS).unwrap();

        Ok(Self {
            db_url: config.db_url,
            chain: config.chain,
        })
    }

    fn establish_connection(&self) -> PgConnection {
        let connection =
            PgConnection::establish(&self.db_url).expect("Unable to connect to the database");

        return connection;
    }

    pub async fn get_block_numbers(&self) -> Result<Vec<i64>> {
        let mut connection = self.establish_connection();

        let blocks = blocks_table
            .select(blocks::number)
            .filter(blocks::chain.eq(self.chain.clone()))
            .load::<i64>(&mut connection);

        match blocks {
            Ok(blocks) => Ok(blocks),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn store_blocks_and_txs(
        &self,
        blocks: Vec<DatabaseBlock>,
        txs: Vec<DatabaseTx>,
        receipts: Vec<DatabaseTxReceipt>,
        logs: Vec<DatabaseTxLogs>,
        contract_creations: Vec<DatabaseContractCreation>,
        contract_interactions: Vec<DatabaseContractInteraction>,
        token_transfers: Vec<DatabaseTokenTransfers>,
    ) {
        let mut stores: Vec<BoxFuture<_>> = vec![];

        let mut log = String::from("Inserted: ");

        if blocks.len() > 0 {
            stores.push(Box::pin(self.store_blocks(&blocks)));
            log.push_str(&format!("blocks({}) ", blocks.len()));
        }

        if txs.len() > 0 {
            stores.push(Box::pin(self.store_txs(&txs)));
            log.push_str(&format!("txs({}) ", txs.len()));
        }

        if receipts.len() > 0 {
            stores.push(Box::pin(self.store_tx_receipts(&receipts)));
            log.push_str(&format!("receipts({}) ", receipts.len()));
        }

        if logs.len() > 0 {
            stores.push(Box::pin(self.store_tx_logs(&logs)));
            log.push_str(&format!("logs({}) ", logs.len()));
        }

        if contract_creations.len() > 0 {
            stores.push(Box::pin(self.store_contract_creations(&contract_creations)));
            log.push_str(&format!(
                "contract_creations({}) ",
                contract_creations.len()
            ));
        }

        if contract_interactions.len() > 0 {
            stores.push(Box::pin(
                self.store_contract_interactions(&contract_interactions),
            ));
            log.push_str(&format!(
                "contract_interactions({}) ",
                contract_interactions.len()
            ));
        }

        if token_transfers.len() > 0 {
            stores.push(Box::pin(self.store_token_transfers(&token_transfers)));
            log.push_str(&format!("token_transfers({})", token_transfers.len()));
        }

        join_all(stores).await;

        info!("{}", log);
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

        Ok(())
    }

    async fn store_tx_logs(&self, logs: &Vec<DatabaseTxLogs>) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in logs.chunks(500) {
            diesel::insert_into(schema::logs::dsl::logs)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store logs in the database");
        }

        Ok(())
    }

    async fn store_contract_creations(
        &self,
        contract_creations: &Vec<DatabaseContractCreation>,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in contract_creations.chunks(500) {
            diesel::insert_into(schema::contract_creations::dsl::contract_creations)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store contract creations in the database");
        }

        Ok(())
    }

    async fn store_contract_interactions(
        &self,
        contract_interactions: &Vec<DatabaseContractInteraction>,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in contract_interactions.chunks(500) {
            diesel::insert_into(schema::contract_interactions::dsl::contract_interactions)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store contract interactions in the database");
        }

        Ok(())
    }

    async fn store_token_transfers(
        &self,
        token_transfers: &Vec<DatabaseTokenTransfers>,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        for chunk in token_transfers.chunks(500) {
            diesel::insert_into(schema::token_transfers::dsl::token_transfers)
                .values(chunk)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store token transfers in the database");
        }

        Ok(())
    }
}
