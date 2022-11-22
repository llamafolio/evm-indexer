use clap::Parser;
use dotenv::dotenv;
use evm_indexer::{
    config::{Config, DEFAULT_AMOUNT_OF_WORKERS, DEFAULT_FETCHER_BATCH_SIZE},
    db::Database,
    rpc::Rpc,
};
use log::*;
use simple_logger::SimpleLogger;

#[derive(Parser, Debug)]
#[command(
    name = "EVM Indexer",
    about = "Minimalistc EVM chain compatible indexer."
)]
pub struct Args {
    #[arg(short, long, help = "Start log with debug", default_value_t = false)]
    pub debug: bool,
    #[arg(
        short, long,
        help = "Amount of workers to fetch blocks",
        default_value_t = DEFAULT_AMOUNT_OF_WORKERS
    )]
    pub workers: usize,
    #[arg(short, long, help = "Initial block to fetch from", default_value_t = 0)]
    pub initial_block: usize,
    #[arg(
        short, long,
        help = "Amount of blocks to fetch by batch",
        default_value_t = DEFAULT_FETCHER_BATCH_SIZE
    )]
    pub batch_size: usize,
}

#[tokio::main(worker_threads = 2)]
async fn main() {
    let log = SimpleLogger::new().with_level(LevelFilter::Info);
    let args = Args::parse();

    if args.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    dotenv().ok();

    info!("Starting EVM Indexer");

    let config = Config::new();

    let db = Database::new(config.clone(), args.initial_block)
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
            rpc.fetch_blocks_range_workers(
                &db,
                last_synced_block + 1,
                last_chain_block,
                args.batch_size,
                args.workers,
            )
            .await;
        }
    });

    rpc.subscribe_heads(&db).await;
}
