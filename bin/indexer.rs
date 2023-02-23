use std::{thread::sleep, time::Duration};

use dotenv::dotenv;
use evm_indexer::{
    chains::chains::Chain,
    configs::indexer_config::EVMIndexerConfig,
    db::{
        db::Database,
        models::models::{
            DatabaseBlock, DatabaseChainIndexedState, DatabaseContract, DatabaseLog,
            DatabaseReceipt, DatabaseTransaction,
        },
    },
    rpc::rpc::Rpc,
};
use futures::future::join_all;
use log::*;
use simple_logger::SimpleLogger;

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let mut config = EVMIndexerConfig::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    info!("Starting EVM Indexer.");

    if !config.reset {
        info!("Syncing chain {}.", config.chain.name.clone());
    }

    let rpc = Rpc::new(&config)
        .await
        .expect("Unable to start RPC client.");

    let db = Database::new(
        config.db_url.clone(),
        config.redis_url.clone(),
        config.chain.clone(),
    )
    .await
    .expect("Unable to start DB connection.");

    if config.recalc_blocks_indexer {
        info!("Updating state of indexed tokens.");
        db.update_indexed_blocks().await.unwrap();
    }

    if !config.reset {
        loop {
            sync_chain(&rpc, &db, &mut config).await;

            sleep(Duration::from_millis(500))
        }
    } else {
        db.delete_indexed_blocks().await.unwrap();
    }
}

async fn sync_chain(rpc: &Rpc, db: &Database, config: &EVMIndexerConfig) {
    let last_block = rpc.get_last_block().await.unwrap();

    let full_block_range = config.start_block..last_block;

    let mut indexed_blocks = db.get_indexed_blocks().await.unwrap();

    let db_state = DatabaseChainIndexedState {
        chain: config.chain.name.to_string(),
        indexed_blocks_amount: indexed_blocks.len() as i64,
    };

    db.update_indexed_blocks_number(&db_state).await.unwrap();

    let missing_blocks: Vec<i64> = full_block_range
        .into_iter()
        .filter(|block| !indexed_blocks.contains(block))
        .collect();

    let total_missing_blocks = missing_blocks.len();

    info!("Syncing {} blocks.", total_missing_blocks);

    let missing_blocks_chunks = missing_blocks.chunks(config.batch_size);

    for missing_blocks_chunk in missing_blocks_chunks {
        let mut work = vec![];

        for block_number in missing_blocks_chunk {
            work.push(fetch_block(&rpc, &block_number, &config.chain))
        }

        let results = join_all(work).await;

        let mut db_blocks: Vec<DatabaseBlock> = Vec::new();
        let mut db_transactions: Vec<DatabaseTransaction> = Vec::new();
        let mut db_receipts: Vec<DatabaseReceipt> = Vec::new();
        let mut db_logs: Vec<DatabaseLog> = Vec::new();
        let mut db_contracts: Vec<DatabaseContract> = Vec::new();

        for result in results {
            match result {
                Some((block, mut transactions, mut receipts, mut logs, mut contracts)) => {
                    db_blocks.push(block);
                    db_transactions.append(&mut transactions);
                    db_receipts.append(&mut receipts);
                    db_logs.append(&mut logs);
                    db_contracts.append(&mut contracts);
                }
                None => continue,
            }
        }

        db.store_data(
            &db_blocks,
            &db_transactions,
            &db_receipts,
            &db_logs,
            &db_contracts,
        )
        .await;

        for block in db_blocks.into_iter() {
            indexed_blocks.insert(block.number);
        }

        db.store_indexed_blocks(&indexed_blocks).await.unwrap();
    }
}

async fn fetch_block(
    rpc: &Rpc,
    block_number: &i64,
    chain: &Chain,
) -> Option<(
    DatabaseBlock,
    Vec<DatabaseTransaction>,
    Vec<DatabaseReceipt>,
    Vec<DatabaseLog>,
    Vec<DatabaseContract>,
)> {
    let block_data = rpc.get_block(block_number).await.unwrap();

    match block_data {
        Some((db_block, mut db_transactions)) => {
            let total_block_transactions = db_transactions.len();

            // Make sure all the transactions are correctly formatted.
            if db_block.transactions != total_block_transactions as i64 {
                warn!(
                    "Missing {} transactions for block {}.",
                    db_block.transactions - total_block_transactions as i64,
                    db_block.number
                );
                return None;
            }

            let mut db_receipts: Vec<DatabaseReceipt> = Vec::new();
            let mut db_logs: Vec<DatabaseLog> = Vec::new();
            let mut db_contracts: Vec<DatabaseContract> = Vec::new();

            if chain.supports_blocks_receipts {
                let receipts_data = rpc.get_block_receipts(block_number).await.unwrap();
                match receipts_data {
                    Some((mut receipts, mut logs, mut contracts)) => {
                        db_receipts.append(&mut receipts);
                        db_logs.append(&mut logs);
                        db_contracts.append(&mut contracts);
                    }
                    None => return None,
                }
            } else {
                for transaction in db_transactions.iter_mut() {
                    let receipt_data = rpc
                        .get_transaction_receipt(transaction.hash.clone())
                        .await
                        .unwrap();

                    match receipt_data {
                        Some((receipt, mut logs, contract)) => {
                            db_receipts.push(receipt);
                            db_logs.append(&mut logs);
                            match contract {
                                Some(contract) => db_contracts.push(contract),
                                None => continue,
                            }
                        }
                        None => continue,
                    }
                }
            }

            if total_block_transactions != db_receipts.len() {
                warn!(
                    "Missing receipts for block {}. Transactions {} receipts {}",
                    db_block.number,
                    total_block_transactions,
                    db_receipts.len()
                );
                return None;
            }

            info!(
                "Found transactions {} receipts {} logs {} and contracts {} for block {}.",
                total_block_transactions,
                db_receipts.len(),
                db_logs.len(),
                db_contracts.len(),
                block_number
            );

            return Some((
                db_block,
                db_transactions,
                db_receipts,
                db_logs,
                db_contracts,
            ));
        }
        None => return None,
    }
}
