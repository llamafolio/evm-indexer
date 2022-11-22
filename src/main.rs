use dotenv::dotenv;
use evm_indexer::{config::Config, db::Database, fetcher, rpc::Rpc};
use log::*;
use simple_logger::SimpleLogger;

#[tokio::main()]
async fn main() {
    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let config = Config::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    dotenv().ok();

    info!("Starting EVM Indexer");

    let db = Database::new(config.clone(), config.initial_block)
        .await
        .expect("Unable to connect to the database");

    let rpc = Rpc::new(config.clone())
        .await
        .expect("Unable to connect to the rpc url");

    // Get the last synced block and compare with the RPC
    let last_synced_block: i64 = db.last_synced_block().await.unwrap();
    let last_chain_block: i64 = rpc.get_last_block().await.unwrap();

    info!("Last DB Synced Block: {}", last_synced_block);
    info!("Last Chain Block: {}", last_chain_block);

    tokio::spawn({
        let rpc = rpc.clone();
        let db = db.clone();
        async move {
            fetcher::fetch_blocks(
                &rpc,
                &db,
                last_synced_block + 1,
                last_chain_block,
                config.batch_size,
                config.workers,
            )
            .await
            .unwrap();
        }
    });

    rpc.subscribe_heads(&db).await;
}
