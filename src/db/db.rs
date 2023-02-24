use std::{cmp::min, collections::HashSet};

use anyhow::Result;
use field_count::FieldCount;
use futures::TryStreamExt;
use log::*;
use redis::Commands;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, QueryBuilder, Row,
};

use crate::chains::chains::Chain;

use super::models::models::{
    DatabaseBlock, DatabaseChainIndexedState, DatabaseContract, DatabaseContractInformation,
    DatabaseLog, DatabaseMethod, DatabaseReceipt, DatabaseTransaction,
};

pub const MAX_DIESEL_PARAM_SIZE: u16 = u16::MAX;

#[derive(Debug, Clone)]
pub struct Database {
    pub chain: Chain,
    pub redis: redis::Client,
    pub db_conn: sqlx::Pool<sqlx::Postgres>,
}

impl Database {
    pub async fn new(db_url: String, redis_url: String, chain: Chain) -> Result<Self> {
        info!("Starting EVM database service");

        let mut connect_options: PgConnectOptions = db_url.parse().unwrap();

        connect_options.disable_statement_logging();

        let db_conn = PgPoolOptions::new()
            .max_connections(500)
            .connect_with(connect_options)
            .await
            .expect("Unable to connect to the database");

        // TODO: db migrations

        let redis = redis::Client::open(redis_url).expect("Unable to connect with Redis server");

        Ok(Self {
            chain,
            redis,
            db_conn,
        })
    }

    pub fn get_connection(&self) -> &sqlx::Pool<sqlx::Postgres> {
        return &self.db_conn;
    }

    pub async fn update_indexed_blocks(&self) -> Result<()> {
        let connection = self.get_connection();

        let mut blocks: HashSet<i64> = HashSet::new();

        let mut rows = sqlx::query("SELECT number FROM blocks WHERE chain = $1")
            .bind(self.chain.name.clone())
            .fetch(connection);

        while let Some(row) = rows.try_next().await.unwrap() {
            let number: i64 = row.try_get("number")?;
            blocks.insert(number);
        }

        self.store_indexed_blocks(&blocks).await.unwrap();

        Ok(())
    }

    pub async fn get_contracts_missing_parsed(&self) -> Result<Vec<DatabaseContract>> {
        let connection = self.get_connection();

        let rows = sqlx::query_as::<_, DatabaseContract>(
            "SELECT * FROM contracts WHERE parsed = true LIMIT 500",
        )
        .fetch_all(connection)
        .await;

        match rows {
            Ok(contracts) => Ok(contracts),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_indexed_blocks(&self) -> Result<HashSet<i64>> {
        let mut connection = self.redis.get_connection().unwrap();

        let keys: Vec<String> = connection
            .keys(format!("{}*", self.chain.name.to_string()))
            .unwrap();

        let mut blocks: HashSet<i64> = HashSet::new();

        for key in keys {
            let chunk_blocks: HashSet<i64> = match connection.get::<String, String>(key) {
                Ok(blocks) => match serde_json::from_str(&blocks) {
                    Ok(deserialized) => deserialized,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            blocks.extend(&chunk_blocks);
        }

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
        let connection = self.get_connection();

        let chunks = get_chunks(blocks.len(), DatabaseBlock::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new("UPSERT INTO blocks (base_fee_per_gas, chain, difficulty, extra_data, gas_limit, gas_used, block_hash, logs_bloom, miner, mix_hash, nonce, number, parent_hash, receipts_root, sha3_uncles, size, state_root, timestamp, total_difficulty, transactions, uncles) ");

            query_builder.push_values(&blocks[start..end], |mut row, block| {
                row.push_bind(block.base_fee_per_gas.clone())
                    .push_bind(block.chain.clone())
                    .push_bind(block.difficulty.clone())
                    .push_bind(block.extra_data.clone())
                    .push_bind(block.gas_limit.clone())
                    .push_bind(block.gas_used.clone())
                    .push_bind(block.block_hash.clone())
                    .push_bind(block.logs_bloom.clone())
                    .push_bind(block.miner.clone())
                    .push_bind(block.mix_hash.clone())
                    .push_bind(block.nonce.clone())
                    .push_bind(block.number)
                    .push_bind(block.parent_hash.clone())
                    .push_bind(block.receipts_root.clone())
                    .push_bind(block.sha3_uncles.clone())
                    .push_bind(block.size)
                    .push_bind(block.state_root.clone())
                    .push_bind(block.timestamp.clone())
                    .push_bind(block.total_difficulty.clone())
                    .push_bind(block.transactions)
                    .push_bind(block.uncles.clone());
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store blocks into database");
        }

        Ok(())
    }

    async fn store_transactions(&self, transactions: &Vec<DatabaseTransaction>) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(transactions.len(), DatabaseTransaction::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new("UPSERT INTO transactions (block_hash, block_number, chain, from_address, gas, gas_price, max_priority_fee_per_gas, max_fee_per_gas, hash, input, method, nonce, timestamp, to_address, transaction_index, transaction_type, value) ");

            query_builder.push_values(&transactions[start..end], |mut row, transaction| {
                row.push_bind(transaction.block_hash.clone())
                    .push_bind(transaction.block_number)
                    .push_bind(transaction.chain.clone())
                    .push_bind(transaction.from_address.clone())
                    .push_bind(transaction.gas.clone())
                    .push_bind(transaction.gas_price.clone())
                    .push_bind(transaction.max_priority_fee_per_gas.clone())
                    .push_bind(transaction.max_fee_per_gas.clone())
                    .push_bind(transaction.hash.clone())
                    .push_bind(transaction.input.clone())
                    .push_bind(transaction.method.clone())
                    .push_bind(transaction.nonce.clone())
                    .push_bind(transaction.timestamp.clone())
                    .push_bind(transaction.to_address.clone())
                    .push_bind(transaction.transaction_index)
                    .push_bind(transaction.transaction_type)
                    .push_bind(transaction.value.clone());
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store transactions into database");
        }

        Ok(())
    }

    async fn store_transactions_receipts(&self, receipts: &Vec<DatabaseReceipt>) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(receipts.len(), DatabaseReceipt::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new("UPSERT INTO receipts (contract_address, cumulative_gas_used, effective_gas_price, gas_used, hash, status) ");

            query_builder.push_values(&receipts[start..end], |mut row, receipt| {
                row.push_bind(receipt.contract_address.clone())
                    .push_bind(receipt.cumulative_gas_used.clone())
                    .push_bind(receipt.effective_gas_price.clone())
                    .push_bind(receipt.gas_used.clone())
                    .push_bind(receipt.hash.clone())
                    .push_bind(receipt.status.clone());
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store receipts into database");
        }

        Ok(())
    }

    async fn store_transactions_logs(&self, logs: &Vec<DatabaseLog>) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(logs.len(), DatabaseLog::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new("UPSERT INTO logs (address, chain, data, erc20_transfers_parsed, hash, log_index, removed, topics) ");

            query_builder.push_values(&logs[start..end], |mut row, log| {
                row.push_bind(log.address.clone())
                    .push_bind(log.chain.clone())
                    .push_bind(log.data.clone())
                    .push_bind(log.erc20_transfers_parsed.clone())
                    .push_bind(log.hash.clone())
                    .push_bind(log.log_index.clone())
                    .push_bind(log.removed.clone())
                    .push_bind(log.topics.clone());
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store logs into database");
        }

        Ok(())
    }

    async fn store_contracts(&self, contracts: &Vec<DatabaseContract>) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(contracts.len(), DatabaseContract::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new(
                "UPSERT INTO contracts (block, chain, contract, creator, hash, parsed, verified) ",
            );

            query_builder.push_values(&contracts[start..end], |mut row, contract| {
                row.push_bind(contract.block.clone())
                    .push_bind(contract.chain.clone())
                    .push_bind(contract.contract.clone())
                    .push_bind(contract.creator.clone())
                    .push_bind(contract.hash.clone())
                    .push_bind(contract.parsed)
                    .push_bind(contract.verified);
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store contracts into database");
        }

        Ok(())
    }

    pub async fn store_contracts_information(
        &self,
        contracts_information: &Vec<DatabaseContractInformation>,
    ) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(
            contracts_information.len(),
            DatabaseContractInformation::field_count(),
        );

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new(
                "UPSERT INTO contracts_information (chain, contract, abi, name, verified) ",
            );

            query_builder.push_values(
                &contracts_information[start..end],
                |mut row, contract_information| {
                    row.push_bind(contract_information.chain.clone())
                        .push_bind(contract_information.contract.clone())
                        .push_bind(contract_information.abi.clone())
                        .push_bind(contract_information.name.clone())
                        .push_bind(contract_information.verified.clone());
                },
            );

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store contracts information into database");
        }

        Ok(())
    }

    pub async fn store_methods(&self, methods: &Vec<DatabaseMethod>) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(methods.len(), DatabaseMethod::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new("UPSERT INTO methods (method, name) ");

            query_builder.push_values(&methods[start..end], |mut row, method| {
                row.push_bind(method.method.clone())
                    .push_bind(method.name.clone());
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store methods into database");
        }

        Ok(())
    }

    pub async fn store_indexed_blocks(&self, blocks: &HashSet<i64>) -> Result<()> {
        let mut connection = self.redis.get_connection().unwrap();

        let blocks_vec: Vec<&i64> = blocks.into_iter().collect();

        let chunks = blocks_vec.chunks(10_000_000);

        for (i, chunk) in chunks.enumerate() {
            let chunk_vec: Vec<&i64> = chunk.to_vec();

            let serialized = serde_json::to_string(&chunk_vec).unwrap();

            let _: () = connection
                .set(format!("{}-{}", self.chain.name.to_owned(), i), serialized)
                .unwrap();
        }

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
        let connection = self.get_connection();

        let query = format!(
            "UPSERT INTO chains_indexed_state (chain, indexed_blocks_amount) VALUES ('{}', {})",
            chain_state.chain.clone(),
            chain_state.indexed_blocks_amount
        );

        QueryBuilder::new(query)
            .build()
            .execute(connection)
            .await
            .expect("Unable to update indexed blocks number");

        Ok(())
    }

    pub async fn update_contracts(&self, contracts: &Vec<DatabaseContract>) -> Result<()> {
        let connection = self.get_connection();

        let chunks = get_chunks(contracts.len(), DatabaseContract::field_count());

        for (start, end) in chunks {
            let mut query_builder = QueryBuilder::new(
                "UPSERT INTO contracts(block, chain, contract, creator, hash, parsed, verified) ",
            );

            query_builder.push_values(&contracts[start..end], |mut row, contract| {
                row.push_bind(contract.block.clone())
                    .push_bind(contract.chain.clone())
                    .push_bind(contract.contract.clone())
                    .push_bind(contract.creator.clone())
                    .push_bind(contract.hash.clone())
                    .push_bind(contract.parsed)
                    .push_bind(contract.verified);
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
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
