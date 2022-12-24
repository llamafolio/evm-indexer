pub mod models;
mod schema;

use std::cmp::min;
use std::collections::HashSet;

use anyhow::Result;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel_migrations::*;
use field_count::FieldCount;
use log::*;

use crate::chains::Chain;
use crate::config::Config;

use self::models::DatabaseBlock;

use self::models::DatabaseContractABI;
use self::models::DatabaseContractAdapter;
use self::models::DatabaseContractCreation;
use self::models::DatabaseContractInteraction;
use self::models::DatabaseExcludedToken;
use self::models::DatabaseMethodID;
use self::models::DatabaseState;
use self::models::DatabaseToken;
use self::models::DatabaseTokenTransfers;
use self::models::DatabaseTx;
use self::models::DatabaseTxLogs;
use self::models::DatabaseTxNoReceipt;
use self::models::DatabaseTxReceipt;
use self::schema::blocks;
use self::schema::blocks::table as blocks_table;
use self::schema::contract_abis;
use self::schema::contract_abis::table as contract_abis_table;
use self::schema::contract_creations;
use self::schema::contract_creations::table as contract_creations_table;
use self::schema::excluded_tokens;
use self::schema::excluded_tokens::table as excluded_tokens_table;
use self::schema::token_transfers;
use self::schema::token_transfers::table as token_transfers_table;
use self::schema::txs_no_receipt;
use self::schema::txs_no_receipt::table as txs_no_receipt_table;

use self::schema::tokens;
use self::schema::tokens::table as tokens_table;

use self::schema::state;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub const MAX_DIESEL_PARAM_SIZE: u16 = u16::MAX;
pub struct State {
    pub id: String,
    pub last_block: i64,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub db_url: String,
    pub chain: Chain,
    pub receipts_fetch_batch: usize,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Initializing Database");

        let mut connection =
            PgConnection::establish(&config.db_url).expect("Unable to connect to the database");

        connection.run_pending_migrations(MIGRATIONS).unwrap();

        let mut limit = 10000;

        if config.remote_rpc != String::from("") {
            limit = 200;
        }

        Ok(Self {
            db_url: config.db_url.to_string(),
            chain: config.chain,
            receipts_fetch_batch: limit,
        })
    }

    fn establish_connection(&self) -> PgConnection {
        let connection =
            PgConnection::establish(&self.db_url).expect("Unable to connect to the database");

        return connection;
    }

    pub async fn get_missing_receipts_txs(&self) -> Result<Vec<String>> {
        let mut connection = self.establish_connection();

        let txs: Vec<String> = txs_no_receipt_table
            .select(txs_no_receipt::hash)
            .filter(txs_no_receipt::chain.eq(self.chain.name.to_string()))
            .limit(self.receipts_fetch_batch as i64)
            .load::<String>(&mut connection)
            .unwrap();

        Ok(txs)
    }

    pub async fn get_tokens_missing_data(&self) -> Result<Vec<String>> {
        let mut connection = self.establish_connection();

        let token_addresses = token_transfers_table
            .select(token_transfers::token)
            .filter(token_transfers::chain.eq(self.chain.name.to_string()))
            .distinct()
            .load::<String>(&mut connection);

        let token_transfers_addresses = match token_addresses {
            Ok(token_addresses) => token_addresses,
            Err(_) => Vec::new(),
        };

        let tokens_stored = tokens_table
            .select(tokens::address)
            .filter(tokens::chain.eq(self.chain.name.to_string()))
            .distinct()
            .load::<String>(&mut connection);

        let token_stored_addresses: HashSet<String> = match tokens_stored {
            Ok(token_addresses) => HashSet::from_iter(token_addresses),
            Err(_) => HashSet::new(),
        };

        let excluded_tokens_stored = excluded_tokens_table
            .select(excluded_tokens::address)
            .filter(excluded_tokens::chain.eq(self.chain.name.to_string()))
            .distinct()
            .load::<String>(&mut connection);

        let excluded_tokens_stored_addresses: HashSet<String> = match excluded_tokens_stored {
            Ok(token_addresses) => HashSet::from_iter(token_addresses),
            Err(_) => HashSet::new(),
        };

        let missing_tokens: Vec<String> = token_transfers_addresses
            .into_iter()
            .filter(|n| {
                !token_stored_addresses.contains(n) && !excluded_tokens_stored_addresses.contains(n)
            })
            .collect();

        Ok(missing_tokens)
    }

    pub async fn get_block_numbers(&self) -> Result<Vec<i64>> {
        let mut connection = self.establish_connection();

        let blocks = blocks_table
            .select(blocks::number)
            .filter(blocks::chain.eq(self.chain.name.to_string()))
            .load::<i64>(&mut connection);

        match blocks {
            Ok(blocks) => Ok(blocks),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_created_contracts(&self) -> Result<Vec<String>> {
        let mut connection = self.establish_connection();

        let contracts = contract_creations_table
            .select(contract_creations::contract)
            .filter(contract_creations::chain.eq(self.chain.name.to_string()))
            .load::<String>(&mut connection);

        match contracts {
            Ok(contracts) => Ok(contracts),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_contracts_with_abis(&self) -> Result<Vec<String>> {
        let mut connection = self.establish_connection();

        let contracts = contract_abis_table
            .select(contract_abis::address)
            .filter(contract_abis::chain.eq(self.chain.name.to_string()))
            .load::<String>(&mut connection);

        match contracts {
            Ok(contracts) => Ok(contracts),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_contract_abi(&self, contract: &String) -> Result<Option<String>> {
        let mut connection = self.establish_connection();

        let abi = contract_abis_table
            .select(contract_abis::abi)
            .filter(contract_abis::address_with_chain.eq(format!(
                "{}-{}",
                contract,
                self.chain.name.to_string()
            )))
            .first::<Option<String>>(&mut connection);

        match abi {
            Ok(interactions) => Ok(interactions),
            Err(_) => Ok(None),
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
        let mut log = String::new();

        if txs.len() > 0 {
            self.store_txs(&txs).await.unwrap();
            log.push_str(&format!("txs({})", txs.len()));
        }

        if receipts.len() > 0 {
            self.store_tx_receipts(&receipts).await.unwrap();
            log.push_str(&format!(" receipts({})", receipts.len()));
        }

        if logs.len() > 0 {
            self.store_tx_logs(&logs).await.unwrap();
            log.push_str(&format!(" logs({})", logs.len()));
        }

        if contract_creations.len() > 0 {
            self.store_contract_creations(&contract_creations)
                .await
                .unwrap();

            log.push_str(&format!(
                " contract_creations({})",
                contract_creations.len()
            ));
        }

        if contract_interactions.len() > 0 {
            self.store_contract_interactions(&contract_interactions)
                .await
                .unwrap();
            log.push_str(&format!(
                " contract_interactions({})",
                contract_interactions.len()
            ));
        }

        if token_transfers.len() > 0 {
            self.store_token_transfers(&token_transfers).await.unwrap();
            log.push_str(&format!(" token_transfers({})", token_transfers.len()));
        }

        if blocks.len() > 0 {
            self.store_blocks(&blocks).await.unwrap();
            log.push_str(&format!(" blocks({})", blocks.len()));
        }

        self.update_chain_state().await.unwrap();

        if log.len() > 0 {
            info!(
                "Inserted: {} for chain {}",
                log,
                self.chain.name.to_string()
            );
        }
    }

    async fn store_blocks(&self, blocks: &Vec<DatabaseBlock>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(blocks.len(), DatabaseBlock::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::blocks::dsl::blocks)
                .values(&blocks[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store blocks into database");
        }

        Ok(())
    }

    async fn store_txs(&self, txs: &Vec<DatabaseTx>) -> Result<()> {
        let mut connection = self.establish_connection();
        info!("Connected");

        let chunks = get_chunks(txs.len(), DatabaseTx::field_count());

        info!("Chunks split");

        for (start, end) in chunks {
            diesel::insert_into(schema::txs::dsl::txs)
                .values(&txs[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store txs into database");
        }

        info!("Query finished split");

        Ok(())
    }

    async fn store_tx_receipts(&self, tx_receipts: &Vec<DatabaseTxReceipt>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(tx_receipts.len(), DatabaseTxReceipt::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::txs_receipts::dsl::txs_receipts)
                .values(&tx_receipts[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store tx_receipts into database");
        }

        Ok(())
    }

    async fn store_tx_logs(&self, logs: &Vec<DatabaseTxLogs>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(logs.len(), DatabaseTxLogs::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::logs::dsl::logs)
                .values(&logs[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store logs into database");
        }

        Ok(())
    }

    async fn store_contract_creations(
        &self,
        contract_creations: &Vec<DatabaseContractCreation>,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(
            contract_creations.len(),
            DatabaseContractCreation::field_count(),
        );

        for (start, end) in chunks {
            diesel::insert_into(schema::contract_creations::dsl::contract_creations)
                .values(&contract_creations[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store contract creations into database");
        }

        Ok(())
    }

    async fn store_contract_interactions(
        &self,
        contract_interactions: &Vec<DatabaseContractInteraction>,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(
            contract_interactions.len(),
            DatabaseContractInteraction::field_count(),
        );

        for (start, end) in chunks {
            diesel::insert_into(schema::contract_interactions::dsl::contract_interactions)
                .values(&contract_interactions[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store contract interactions into database");
        }

        Ok(())
    }

    async fn store_token_transfers(
        &self,
        token_transfers: &Vec<DatabaseTokenTransfers>,
    ) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(token_transfers.len(), DatabaseTokenTransfers::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::token_transfers::dsl::token_transfers)
                .values(&token_transfers[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store token transfers into database");
        }

        Ok(())
    }

    pub async fn store_tokens(&self, tokens: &Vec<DatabaseToken>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(tokens.len(), DatabaseToken::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::tokens::dsl::tokens)
                .values(&tokens[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store tokens into database");
        }

        Ok(())
    }

    pub async fn store_excluded_tokens(&self, tokens: &Vec<DatabaseExcludedToken>) -> Result<()> {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(tokens.len(), DatabaseExcludedToken::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::excluded_tokens::dsl::excluded_tokens)
                .values(&tokens[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store excluded tokens into database");
        }

        Ok(())
    }

    pub async fn store_txs_no_receipt(&self, txs: &Vec<DatabaseTxNoReceipt>) {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(txs.len(), DatabaseTxNoReceipt::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::txs_no_receipt::dsl::txs_no_receipt)
                .values(&txs[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store transactions with no receipt into database");
        }
    }

    pub async fn store_contract_abi(&self, contract_abi: &DatabaseContractABI) {
        let mut connection = self.establish_connection();

        diesel::insert_into(schema::contract_abis::dsl::contract_abis)
            .values(contract_abi)
            .on_conflict(contract_abis::address_with_chain)
            .do_update()
            .set((
                schema::contract_abis::dsl::abi.eq(&contract_abi.abi),
                schema::contract_abis::dsl::verified.eq(&contract_abi.verified),
            ))
            .execute(&mut connection)
            .expect("Unable to store contract abis into database");
    }

    pub async fn store_abi_method_ids(&self, method_ids: &Vec<DatabaseMethodID>) {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(method_ids.len(), DatabaseMethodID::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::method_ids::dsl::method_ids)
                .values(&method_ids[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store method ids into database");
        }
    }

    pub async fn store_contract_adapters(&self, adapters: &Vec<DatabaseContractAdapter>) {
        let mut connection = self.establish_connection();

        let chunks = get_chunks(adapters.len(), DatabaseContractAdapter::field_count());

        for (start, end) in chunks {
            diesel::insert_into(schema::contracts_adapters::dsl::contracts_adapters)
                .values(&adapters[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to contract adapters into database");
        }
    }

    async fn update_chain_state(&self) -> Result<()> {
        let mut connection = self.establish_connection();

        let blocks = self.get_block_numbers().await.unwrap();

        let state = DatabaseState {
            chain: self.chain.name.to_string(),
            blocks: blocks.len() as i64,
        };

        diesel::insert_into(schema::state::dsl::state)
            .values(&state)
            .on_conflict(state::chain)
            .do_update()
            .set(schema::state::dsl::blocks.eq(state.blocks))
            .execute(&mut connection)
            .expect("Unable to update chain state");

        Ok(())
    }

    pub async fn delete_no_receipt_txs(&self, txs: &Vec<String>) {
        let mut connection = self.establish_connection();

        for tx in txs {
            diesel::delete(txs_no_receipt_table.filter(txs_no_receipt::hash.eq(tx)))
                .execute(&mut connection)
                .expect("Unable to delete no receipt transactions");
        }
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
