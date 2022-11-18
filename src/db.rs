use anyhow::Result;

use mongodb::{bson::doc, options::ClientOptions, Client, Collection, Database};
use serde::{Deserialize, Serialize};
use web3::{
    types::{Block, Transaction, H160, U256},
    Error,
};

use crate::utils::{
    format_address, format_block, format_bytes, format_hash, format_nonce, format_number,
};

const STATE_COLLECTION: &str = "state";
const STATE_COLLECTION_ID: &str = "sync_state";

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    #[serde(rename = "_id")]
    pub id: String,
    pub last_block: i64,
}

const BLOCKS_COLLECTION: &str = "blocks";

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseBlock {
    pub height: i64,
    #[serde(rename = "_id")]
    pub hash: String,
    pub txs: i64,
    pub timestamp: i64,
    pub size: i64,
    pub nonce: String,
}

impl DatabaseBlock {
    pub fn from_web3_block(block: &Block<Transaction>) -> DatabaseBlock {
        return DatabaseBlock {
            height: block.number.unwrap().as_u64() as i64,
            hash: format_hash(block.hash.unwrap()),
            timestamp: block.timestamp.as_u64() as i64,
            txs: block.transactions.len() as i64,
            size: block.size.unwrap().as_u64() as i64,
            nonce: format_nonce(block.nonce.unwrap()),
        };
    }
}

const TXS_COLLECTION: &str = "txs";

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseTx {
    #[serde(rename = "_id")]
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
    pub fn from_web3_tx(tx: Transaction, block_timestamp: U256) -> DatabaseTx {
        let to_address: String = match tx.to {
            None => format_address(H160::zero()),
            Some(to) => format_address(to),
        };

        let tx_type: i64 = match tx.transaction_type {
            None => 0,
            Some(to) => to.as_u64() as i64,
        };

        let tx_db = DatabaseTx {
            hash: format_hash(tx.hash),
            from_address: format_address(tx.from.unwrap()),
            create_contract: tx.to.is_none(),
            to_address,
            block: tx.block_number.unwrap().as_u64() as i64,
            tx_value: format_number(tx.value),
            timestamp: block_timestamp.as_u64() as i64,
            tx_index: tx.transaction_index.unwrap().as_u64() as i64,
            tx_type,
            data_input: format_bytes(&tx.input),
            gas: format_number(tx.gas),
            gas_price: format_number(tx.gas_price.unwrap()),
        };

        return tx_db;
    }
}

pub struct IndexerDB {
    pub db: Database,
    pub initial_block: usize,
}

impl IndexerDB {
    pub async fn new(db_url: &str, db_name: &str, initial_block: usize) -> Result<Self> {
        log::info!("==> IndexerDB: Initializing IndexerDB");

        let client_options = ClientOptions::parse(db_url).await?;

        let client = Client::with_options(client_options)?;

        let db = client.database(db_name);

        Ok(IndexerDB { db, initial_block })
    }

    pub async fn last_synced_block(&self) -> Result<i64> {
        let collection: Collection<State> = self.db.collection(STATE_COLLECTION);

        let filter = doc! {"_id": STATE_COLLECTION_ID};

        let state_raw = collection.find_one(filter, None).await.unwrap();

        match state_raw {
            None => {
                // If no data, initialize the table
                let initial_state = State {
                    id: String::from(STATE_COLLECTION_ID),
                    last_block: 15990000,
                };

                collection
                    .insert_one(initial_state, None)
                    .await
                    .expect("Unable to write initial state data");

                Ok(15990000)
            }
            Some(state) => Ok(state.last_block),
        }
    }

    pub async fn store_block_batch(
        &self,
        blocks: Vec<Result<serde_json::Value, Error>>,
        update_sync_state: bool,
    ) {
        let mut blocks_db: Vec<DatabaseBlock> = Vec::new();

        let collection: Collection<DatabaseBlock> = self.db.collection(BLOCKS_COLLECTION);

        for block_raw in blocks.iter() {
            let block: Block<Transaction> = format_block(block_raw);

            let block_db = DatabaseBlock::from_web3_block(&block);

            blocks_db.push(block_db);
        }

        collection
            .insert_many(blocks_db, None)
            .await
            .expect("Unable to store block batch");

        log::info!("==> IndexerDB: Stored {} blocks", blocks.len());

        let last_block: Block<Transaction> = format_block(blocks.last().unwrap());

        if update_sync_state {
            self.update_sync_state(last_block.number.unwrap().as_u64() as i64)
                .await;
        }

        self.store_txs_batch(blocks).await;
    }

    pub async fn store_txs_batch(&self, blocks: Vec<Result<serde_json::Value, Error>>) {
        let collection: Collection<DatabaseTx> = self.db.collection(TXS_COLLECTION);

        let mut txs: Vec<DatabaseTx> = Vec::new();

        for block_raw in blocks.iter() {
            let block = format_block(block_raw);

            for tx in block.transactions {
                let tx_db = DatabaseTx::from_web3_tx(tx, block.timestamp);

                txs.push(tx_db);
            }
        }

        let txs_amount = txs.len();

        if txs_amount > 0 {
            collection
                .insert_many(txs, None)
                .await
                .expect("Unable to store txs batch");

            log::info!("==> IndexerDB: Stored {} txs", txs_amount);
        }
    }

    pub async fn update_sync_state(&self, last_block: i64) {
        let collection: Collection<State> = self.db.collection(STATE_COLLECTION);

        let filter = doc! {"_id": STATE_COLLECTION_ID};

        let state = collection.find_one(filter, None).await.unwrap();

        match state {
            None => return,
            Some(mut state) => {
                state.last_block = last_block;

                let filter = doc! {"_id": STATE_COLLECTION_ID};

                collection
                    .replace_one(filter, state, None)
                    .await
                    .expect("Unable to write initial state data");

                log::info!("==> IndexerDB: Updated sync state to block {}", last_block);
            }
        }
    }

    pub async fn store_block(&self, block: Block<Transaction>) {
        log::info!("==> IndexerDB: Storing block {}", block.number.unwrap());

        let block_db: DatabaseBlock = DatabaseBlock::from_web3_block(&block);

        let collection: Collection<DatabaseBlock> = self.db.collection(BLOCKS_COLLECTION);

        collection
            .insert_one(block_db, None)
            .await
            .expect("Unable to store block");

        self.store_block_txs(block).await;
    }

    pub async fn store_block_txs(&self, block: Block<Transaction>) {
        let collection: Collection<DatabaseTx> = self.db.collection(TXS_COLLECTION);

        let mut txs: Vec<DatabaseTx> = Vec::new();

        for tx in block.transactions {
            let tx_db = DatabaseTx::from_web3_tx(tx, block.timestamp);

            txs.push(tx_db);
        }

        let txs_amount = txs.len();

        if txs_amount > 0 {
            collection
                .insert_many(txs, None)
                .await
                .expect("Unable to store txs");

            log::info!("==> IndexerDB: Stored {} txs", txs_amount);
        }
    }
}
