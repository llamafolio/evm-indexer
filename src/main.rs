mod db;
mod rpc;
mod utils;

use dotenv::dotenv;
use log::LevelFilter;
use web3::futures::future::join_all;

use crate::db::IndexerDB;
use crate::rpc::IndexerRPC;
use simple_logger::SimpleLogger;

const DEFAULT_FETCHER_BATCH_SIZE: usize = 1000;
const DEFAULT_AMOUNT_OF_WORKERS: usize = 10;

async fn fetch_blocks_range(
    rpc: &IndexerRPC,
    db: &IndexerDB,
    chunk: &[i64],
    update_sync_state: bool,
) {
    log::info!(
        "==> Main: Procesing chunk from block {} to {}",
        chunk.first().unwrap(),
        chunk.last().unwrap()
    );

    let blocks = rpc.fetch_block_batch(chunk).await.unwrap();

    if blocks.len() > 0 {
        db.store_block_batch(blocks, update_sync_state).await;
    }
}

async fn fetch_blocks_range_workers(
    rpc: &IndexerRPC,
    db: &IndexerDB,
    from: i64,
    to: i64,
    batch_size: usize,
    workers: usize,
) {
    log::info!(
        "==> Main: Fetching block range from {} to {} with batches of {} blocks with {} workers",
        from,
        to,
        batch_size,
        workers
    );

    let full_block_range: Vec<i64> = (from..to).collect();

    for work_chunk in full_block_range.chunks(batch_size * workers) {
        let mut works = vec![];

        let chunks = work_chunk.chunks(batch_size);

        let chunks_size = chunks.len();

        for (i, worker_part) in chunks.enumerate() {
            works.push(fetch_blocks_range(
                rpc,
                db,
                worker_part,
                i == chunks_size - 1,
            ));
        }

        join_all(works).await;
    }
}

#[tokio::main(worker_threads = 2)]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    dotenv().ok();

    log::info!("Starting EVM Indexer");

    // Load .env variables
    let db_url = std::env::var("DB_URL").expect("DB_URL must be set.");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME must be set.");
    let rpc_ws_url = std::env::var("RPC_WS_URL").expect("RPC_WS_URL must be set.");
    let rpc_http_url = std::env::var("RPC_HTTPS_URL").expect("RPC_HTTPS_URL must be set.");

    let db = IndexerDB::new(&db_url, &db_name)
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

    tokio::join!(
        rpc.subscribe_heads(&db),
        fetch_blocks_range_workers(
            &rpc,
            &db,
            last_synced_block + 1,
            last_chain_block,
            DEFAULT_FETCHER_BATCH_SIZE,
            DEFAULT_AMOUNT_OF_WORKERS
        ),
    );
}
