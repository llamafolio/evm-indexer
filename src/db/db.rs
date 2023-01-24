use std::cmp::min;
use std::collections::HashSet;

use anyhow::Result;
use diesel::prelude::*;
use diesel::{Connection, PgConnection};
use diesel_migrations::*;
use field_count::FieldCount;
use log::*;
use redis::Commands;

use crate::chains::chains::Chain;

use super::models::models::{
    DatabaseAbi, DatabaseBlock, DatabaseChainIndexedState, DatabaseContract, DatabaseLog,
    DatabaseMethod, DatabaseReceipt, DatabaseTransaction,
};
use super::schema::*;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub const MAX_DIESEL_PARAM_SIZE: u16 = u16::MAX;

#[derive(Debug, Clone)]
pub struct EVMDatabase {
    pub db_url: String,
    pub chain: Chain,
    pub redis: redis::Client,
}

impl EVMDatabase {
    pub async fn new(db_url: String, redis_url: String, chain: Chain) -> Result<Self> {
        info!("Starting EVM database service");

        let mut connection =
            PgConnection::establish(&db_url).expect("Unable to connect to the database");

        connection.run_pending_migrations(MIGRATIONS).unwrap();

        let redis = redis::Client::open(redis_url).expect("Unable to connect with Redis server");

        Ok(Self {
            db_url,
            chain,
            redis,
        })
    }

    pub fn establish_connection(&self) -> PgConnection {
        let connection =
            PgConnection::establish(&self.db_url).expect("Unable to connect to the database");

        return connection;
    }

    pub async fn get_contracts(&self) -> Result<Vec<DatabaseContract>> {
        let mut connection = self.establish_connection();

        let contracts = contracts::dsl::contracts
            .select(contracts::all_columns)
            .filter(contracts::parsed.eq(false))
            .limit(500)
            .load::<DatabaseContract>(&mut connection);

        match contracts {
            Ok(contracts) => Ok(contracts),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_indexed_blocks(&self) -> Result<HashSet<i64>> {
        let mut connection = self.redis.get_connection().unwrap();

        let blocks: HashSet<i64> =
            match connection.get::<String, String>(self.chain.name.to_string()) {
                Ok(blocks) => match serde_json::from_str(&blocks) {
                    Ok(deserialized) => deserialized,
                    Err(_) => HashSet::new(),
                },
                Err(_) => HashSet::new(),
            };

        Ok(blocks)
    }

    pub async fn store_data(
        &self,
        blocks: &Vec<DatabaseBlock>,
        transactions: &Vec<DatabaseTransaction>,
        receipts: &Vec<DatabaseReceipt>,
        logs: &Vec<DatabaseLog>,
        contracts: &Vec<DatabaseContract>,
    ) {
        if contracts.len() > 0 {
            self.store_contracts(&contracts).await.unwrap();
        }

        if transactions.len() > 0 {
            self.store_transactions(&transactions).await.unwrap();
        }

        if receipts.len() > 0 {
            self.store_transactions_receipts(&receipts).await.unwrap();
        }

        if logs.len() > 0 {
            self.store_transactions_logs(&logs).await.unwrap();
        }

        if blocks.len() > 0 {
            self.store_blocks(&blocks).await.unwrap();
        }

        info!(
            "Inserted: blocks ({}) transactions ({}) receipts ({}) logs ({}) contracts ({}) for chain {}",
            blocks.len(),
            transactions.len(),
            receipts.len(),
            logs.len(),
            contracts.len(),
            self.chain.name.clone()
        );
    }

    async fn store_blocks(&self, blocks: &Vec<DatabaseBlock>) -> Result<()> {
        let mut connection = self.establish_connection();

        diesel::insert_into(blocks::dsl::blocks)
            .values(blocks)
            .on_conflict_do_nothing()
            .execute(&mut connection)
            .expect("Unable to store blocks into database");

        Ok(())
    }

    async fn store_transactions(&self, transactions: &Vec<DatabaseTransaction>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(transactions.len(), DatabaseTransaction::field_count());

        for (start, end) in chunks {
            diesel::insert_into(transactions::dsl::transactions)
                .values(&transactions[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store transactions into database");
        }

        Ok(())
    }

    async fn store_transactions_receipts(&self, receipts: &Vec<DatabaseReceipt>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(receipts.len(), DatabaseReceipt::field_count());

        for (start, end) in chunks {
            diesel::insert_into(receipts::dsl::receipts)
                .values(&receipts[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store receipts into database");
        }

        Ok(())
    }

    async fn store_transactions_logs(&self, logs: &Vec<DatabaseLog>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(logs.len(), DatabaseLog::field_count());

        for (start, end) in chunks {
            diesel::insert_into(logs::dsl::logs)
                .values(&logs[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store logs into database");
        }

        Ok(())
    }

    async fn store_contracts(&self, contracts: &Vec<DatabaseContract>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(contracts.len(), DatabaseContract::field_count());

        for (start, end) in chunks {
            diesel::insert_into(contracts::dsl::contracts)
                .values(&contracts[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store contracts into database");
        }

        Ok(())
    }

    pub async fn store_abis(&self, abis: &Vec<DatabaseAbi>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(abis.len(), DatabaseAbi::field_count());

        for (start, end) in chunks {
            diesel::insert_into(abis::dsl::abis)
                .values(&abis[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store abis into database");
        }

        Ok(())
    }

    pub async fn store_methods(&self, methods: &Vec<DatabaseMethod>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(methods.len(), DatabaseAbi::field_count());

        for (start, end) in chunks {
            diesel::insert_into(methods::dsl::methods)
                .values(&methods[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store methods into database");
        }

        Ok(())
    }

    pub async fn store_indexed_blocks(&self, blocks: &HashSet<i64>) -> Result<()> {
        let mut connection = self.redis.get_connection().unwrap();

        let serialized = serde_json::to_string(blocks).unwrap();

        let _: () = connection
            .set(self.chain.name.to_string(), serialized)
            .unwrap();

        self.update_indexed_blocks_number(&DatabaseChainIndexedState {
            chain: self.chain.name.to_string(),
            indexed_blocks_amount: blocks.len() as i64,
        })
        .await
        .unwrap();

        Ok(())
    }

    pub async fn update_indexed_blocks_number(
        &self,
        chain_state: &DatabaseChainIndexedState,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        diesel::insert_into(chains_indexed_state::table)
            .values(chain_state)
            .on_conflict(chains_indexed_state::dsl::chain)
            .do_update()
            .set(chains_indexed_state::indexed_blocks_amount.eq(chain_state.indexed_blocks_amount))
            .execute(&mut connection)
            .expect("Unable to update indexed blocks number");

        Ok(())
    }

    pub async fn update_contracts(&self, contracts: &Vec<DatabaseContract>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(contracts.len(), DatabaseContract::field_count());

        for (start, end) in chunks {
            diesel::insert_into(contracts::dsl::contracts)
                .values(&contracts[start..end])
                .on_conflict(contracts::hash)
                .do_update()
                .set(contracts::parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update contracts into database");
        }

        Ok(())
    }

    pub async fn delete_indexed_blocks(&self) -> Result<()> {
        let mut connection = self.redis.get_connection().unwrap();

        let _: () = connection.del(self.chain.name.to_string()).unwrap();

        Ok(())
    }
}

/// Ref: https://github.com/aptos-labs/aptos-core/blob/main/crates/indexer/src/database.rs#L32
/// Given diesel has a limit of how many parameters can be inserted in a single operation (u16::MAX)
/// we may need to chunk an array of items based on how many columns are in the table.
/// This function returns boundaries of chunks in the form of (start_index, end_index)
pub fn get_chunks(num_items_to_insert: usize, column_count: usize) -> Vec<(usize, usize)> {
    let max_item_size = MAX_DIESEL_PARAM_SIZE as usize / column_count;
    let mut chunk: (usize, usize) = (0, min(num_items_to_insert, max_item_size));
    let mut chunks = vec![chunk];
    while chunk.1 != num_items_to_insert {
        chunk = (
            chunk.0 + max_item_size,
            min(num_items_to_insert, chunk.1 + max_item_size),
        );
        chunks.push(chunk);
    }
    chunks
}
