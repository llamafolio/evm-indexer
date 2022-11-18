use anyhow::Result;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::{Client, NoTls};
use web3::{
    types::{Block, Transaction, H160},
    Error,
};

use crate::utils::{
    format_address, format_block, format_bytes, format_hash, format_nonce, format_number,
};

#[derive(PostgresMapper)]
#[pg_mapper(table = "state")]
pub struct State {
    pub id: String,
    pub last_block: i64,
}

const CREATE_STATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS sync_state (
    id VARCHAR NOT NULL UNIQUE,
    last_block BIGINT
  ); 
";

#[derive(PostgresMapper)]
#[pg_mapper(table = "blocks")]
pub struct DatabaseBlock {
    pub height: i64,
    pub hash: String,
    pub txs: i64,
    pub timestamp: i64,
    pub size: i64,
    pub nonce: String,
}

impl DatabaseBlock {
    pub fn to_values(&self) -> String {
        return format!(
            "({},{},{},{},{},{})",
            self.height, self.hash, self.txs, self.timestamp, self.size, self.nonce
        );
    }
}

const CREATE_BLOCKS_TABLE: &str = "CREATE TABLE IF NOT EXISTS blocks (
        height BIGINT UNIQUE,
        hash VARCHAR,
        txs BIGINT,
        timestamp BIGINT,
        size BIGINT,
        nonce VARCHAR
  ); 
";

#[derive(PostgresMapper)]
#[pg_mapper(table = "txs")]
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

impl DatabaseTx {
    pub fn to_values(&self) -> String {
        return format!(
            "({},{},{},{},{},{},{},{},{},{},{}, {})",
            self.hash,
            self.from_address,
            self.to_address,
            self.create_contract,
            self.block,
            self.tx_value,
            self.timestamp,
            self.tx_index,
            self.tx_type,
            self.data_input,
            self.gas,
            self.gas_price
        );
    }
}

const CREATE_TXS_TABLE: &str = "CREATE TABLE IF NOT EXISTS txs (
    hash VARCHAR UNIQUE,
    from_address VARCHAR,
    to_address VARCHAR,
    create_contract BOOL,
    block BIGINT,
    tx_value VARCHAR,
    timestamp BIGINT,
    tx_index BIGINT,
    tx_type BIGINT,
    data_input VARCHAR,
    gas VARCHAR,
    gas_price VARCHAR
  ); 
";

pub struct IndexerDB {
    pub db: Client,
}

impl IndexerDB {
    pub async fn new(db_url: &str) -> Result<Self> {
        log::info!("==> IndexerDB: Initializing IndexerDB");

        let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // Create tables if required
        client
            .query(CREATE_STATE_TABLE, &[])
            .await
            .expect("Unable to run sync_state creation query");

        client
            .query(CREATE_BLOCKS_TABLE, &[])
            .await
            .expect("Unable to run blocks creation query");

        client
            .query(CREATE_TXS_TABLE, &[])
            .await
            .expect("Unable to run txs creation query");

        Ok(IndexerDB { db: client })
    }

    pub async fn last_synced_block(&self) -> Result<i64> {
        let query = &self
            .db
            .query("SELECT * from sync_state", &[])
            .await
            .unwrap();

        // Get the first row to fetch the data
        let row = query.get(0);

        match row {
            None => {
                // If no data, initialize the table
                let _ = &self
                    .db
                    .query(
                        "INSERT INTO sync_state(id, last_block) VALUES ('sync_state', 100000)",
                        &[],
                    )
                    .await
                    .expect("Unable to write initial state data");

                Ok(100000)
            }
            Some(row) => {
                let state = State::from_row_ref(row).unwrap();
                Ok(state.last_block)
            }
        }
    }

    pub async fn store_block_batch(&self, blocks: Vec<Result<serde_json::Value, Error>>) {
        let mut query: String =
            String::from("INSERT INTO blocks (height, hash, txs, timestamp, size, nonce) VALUES ");

        for block_raw in blocks.iter() {
            let block: Block<Transaction> = format_block(block_raw);

            let block_db: DatabaseBlock = DatabaseBlock {
                height: block.number.unwrap().as_u64() as i64,
                hash: format_hash(block.hash.unwrap()),
                txs: block.transactions.len() as i64,
                timestamp: block.timestamp.as_u64() as i64,
                size: block.size.unwrap().as_u64() as i64,
                nonce: format_nonce(block.nonce.unwrap()),
            };

            query.push_str(&block_db.to_values());
            query.push_str(&",");
        }

        // Remove the last comma
        query.pop();

        query.push_str(&"ON CONFLICT (height) DO NOTHING;");

        let _ = &self
            .db
            .query(&query, &[])
            .await
            .expect("Unable to store block batch");

        log::info!("==> IndexerDB: Stored {} blocks", blocks.len());

        let last_block: Block<Transaction> = format_block(blocks.last().unwrap());

        self.update_sync_state(last_block.number.unwrap().as_u64() as i64)
            .await;

        self.store_txs_batch(blocks).await;
    }

    pub async fn store_txs_batch(&self, blocks: Vec<Result<serde_json::Value, Error>>) {
        let mut query: String = String::from(
            "INSERT INTO txs (hash, from_address, to_address, create_contract, block, tx_value, timestamp, tx_index, tx_type, data_input, gas, gas_price) VALUES "
        );

        let mut count: usize = 0;

        for block_raw in blocks.iter() {
            let block = format_block(block_raw);

            for tx in block.transactions {
                let to_address: String = match tx.to {
                    None => format_address(H160::zero()),
                    Some(to) => format_address(to),
                };

                let tx_db = DatabaseTx {
                    hash: format_hash(tx.hash),
                    from_address: format_address(tx.from.unwrap()),
                    create_contract: tx.to.is_none(),
                    to_address,
                    block: tx.block_number.unwrap().as_u64() as i64,
                    tx_value: format_number(tx.value),
                    timestamp: block.timestamp.as_u64() as i64,
                    tx_index: tx.transaction_index.unwrap().as_u64() as i64,
                    tx_type: tx.transaction_type.unwrap().as_u64() as i64,
                    data_input: format_bytes(&tx.input),
                    gas: format_number(tx.gas),
                    gas_price: format_number(tx.gas_price.unwrap()),
                };

                query.push_str(&tx_db.to_values());
                query.push_str(&",");

                count += 1;
            }
        }

        query.pop();

        query.push_str(&"ON CONFLICT (hash) DO NOTHING;");

        if count > 0 {
            // Remove the last comma

            let _ = &self
                .db
                .query(&query, &[])
                .await
                .expect("Unable to store txs batch");

            log::info!("==> IndexerDB: Stored {} txs", count);
        }
    }

    pub async fn update_sync_state(&self, last_block: i64) {
        let query = format!(
            "UPDATE sync_state SET last_block = {} WHERE id = 'sync_state' ",
            last_block
        );

        let _ = &self
            .db
            .query(&query, &[])
            .await
            .expect("Unable to update last block sync state");

        log::info!("==> IndexerDB: Updated sync state to block {}", last_block);
    }

    pub async fn store_block(&self, block: Block<Transaction>) {
        log::info!("==> IndexerDB: Storing block {}", block.number.unwrap());

        let block_db: DatabaseBlock = DatabaseBlock {
            height: block.number.unwrap().as_u64() as i64,
            hash: format_hash(block.hash.unwrap()),
            txs: block.transactions.len() as i64,
            timestamp: block.timestamp.as_u64() as i64,
            size: block.size.unwrap().as_u64() as i64,
            nonce: format_nonce(block.nonce.unwrap()),
        };

        let _ = &self
            .db
            .query(
                "INSERT INTO blocks (height, hash, txs, timestamp, size, nonce) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (height) DO NOTHING;",
                &[&block_db.height, &block_db.hash, &block_db.txs, &block_db.timestamp, &block_db.size, &block_db.nonce],
            )
            .await
            .expect("Unable to store block");

        self.store_txs(block).await;
    }

    pub async fn store_txs(&self, block: Block<Transaction>) {
        let mut query: String = String::from(
            "INSERT INTO txs (hash, from_address, to_address, create_contract, block, tx_value, timestamp, tx_index, tx_type, data_input, gas, gas_price) VALUES "
        );

        let mut count: usize = 0;

        for tx in block.transactions {
            let to_address: String = match tx.to {
                None => format_address(H160::zero()),
                Some(to) => format_address(to),
            };

            let tx_db = DatabaseTx {
                hash: format_hash(tx.hash),
                from_address: format_address(tx.from.unwrap()),
                create_contract: tx.to.is_none(),
                to_address,
                block: tx.block_number.unwrap().as_u64() as i64,
                tx_value: format_number(tx.value),
                timestamp: block.timestamp.as_u64() as i64,
                tx_index: tx.transaction_index.unwrap().as_u64() as i64,
                tx_type: tx.transaction_type.unwrap().as_u64() as i64,
                data_input: format_bytes(&tx.input),
                gas: format_number(tx.gas),
                gas_price: format_number(tx.gas_price.unwrap()),
            };

            query.push_str(&tx_db.to_values());
            query.push_str(&",");

            count += 1;
        }

        query.pop();

        query.push_str(&"ON CONFLICT (hash) DO NOTHING;");

        if count > 0 {
            // Remove the last comma

            let _ = &self
                .db
                .query(&query, &[])
                .await
                .expect("Unable to store txs");

            log::info!("==> IndexerDB: Stored {} txs", count);
        }
    }
}
