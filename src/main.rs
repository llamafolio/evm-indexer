mod db;
mod rpc;
mod utils;

use dotenv::dotenv;
use log::LevelFilter;

use crate::db::IndexerDB;
use crate::rpc::IndexerRPC;
use simple_logger::SimpleLogger;

const DEFAULT_FETCHER_BATCH_SIZE: usize = 5000;

async fn fetch_blocks_range(rpc: IndexerRPC, db: IndexerDB, from: i64, to: i64, batch_size: usize) {
    log::info!(
        "==> Main: Fetching block range from {} to {} with batches of {} blocks",
        from,
        to,
        batch_size
    );

    let blocks_numbers: Vec<i64> = (from..to).collect();

    for chunk in blocks_numbers.chunks(batch_size) {
        log::info!(
            "==> Main: Procesing chunk from block {} to {}",
            chunk.first().unwrap(),
            chunk.last().unwrap()
        );

        let blocks = rpc.fetch_block_batch(chunk).await.unwrap();

        db.store_block_batch(blocks).await
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    dotenv().ok();

    log::info!("Starting EVM Indexer");

    // Load .env variables
    let db_url = std::env::var("DB_URL").expect("DB_URL must be set.");
    let rpc_ws_url = std::env::var("RPC_WS_URL").expect("RPC_WS_URL must be set.");
    let rpc_http_url = std::env::var("RPC_HTTPS_URL").expect("RPC_HTTPS_URL must be set.");

    let db = IndexerDB::new(&db_url)
        .await
        .expect("Unable to connect to the database");
    let rpc = IndexerRPC::new(&rpc_ws_url, &rpc_http_url)
        .await
        .expect("Unable to connect to the rpc url");

    // Get the last synced block and compare with the RPC
    let last_synced_block: i64 = db.last_synced_block().await.unwrap();
    let last_chain_block: i64 = rpc.last_block().await.unwrap();

    log::info!("==> Main: Last DB Synced Block: {}", last_synced_block);
    log::info!("==> Main: Last Chain Block: {}", last_chain_block);

    // Load blocks from last_synced_block to last_chain_block
    /* fetch_blocks_range(
        rpc,
        db,
        last_synced_block + 1,
        last_chain_block,
        DEFAULT_FETCHER_BATCH_SIZE,
    )
    .await; */

    // Subscribe for new blocks
    rpc.subscribe_heads(&db).await;
}
