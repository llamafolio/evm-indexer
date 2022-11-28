pub mod chains;
pub mod config;
pub mod db;
pub mod fetcher;
pub mod rpc;
pub mod utils;

use std::time::Duration;

use dotenv::dotenv;
use log::*;
use simple_logger::SimpleLogger;
use tokio::time::sleep;

use crate::{config::Config, db::Database, rpc::Rpc};

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let config = Config::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    info!("Starting EVM Indexer");

    let db = Database::new(config.clone())
        .await
        .expect("Unable to connect to the database");

    let rpc = Rpc::new(config.clone())
        .await
        .expect("Unable to connect to the rpc url");

    tokio::spawn({
        let rpc = rpc.clone();
        let db = db.clone();
        let config = config.clone();

        async move {
            fetcher::fetch_blocks(&rpc, &db, config).await.unwrap();
        }
    });

    tokio::spawn({
        let rpc = rpc.clone();
        let db = db.clone();
        let config = config.clone();
        async move {
            loop {
                fetcher::fetch_tokens_metadata(&rpc, &db, &config)
                    .await
                    .unwrap();
                sleep(Duration::from_secs(30)).await;
            }
        }
    });

    rpc.subscribe_heads(&db).await;
}
