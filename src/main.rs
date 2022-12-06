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

use crate::{chains::Provider, config::Config, db::Database, rpc::Rpc};

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

    let db = Database::new(&config)
        .await
        .expect("Unable to connect to the database");

    let mut available_providers: Vec<Rpc> = vec![];

    if !config.use_local_rpc {
        let ankr_provider = config.ankr_provider.clone();
        if ankr_provider.is_available(&config.chain) {
            let rpc = Rpc::new(&config, &ankr_provider).await.unwrap();
            available_providers.push(rpc);
        }

        let llamanodes_provider = config.llamanodes_provider.clone();
        if llamanodes_provider.is_available(&config.chain) {
            let rpc = Rpc::new(&config, &llamanodes_provider).await.unwrap();
            available_providers.push(rpc);
        }

        let pokt_provider = config.pokt_provider.clone();
        if pokt_provider.is_available(&config.chain) {
            let rpc = Rpc::new(&config, &pokt_provider).await.unwrap();
            available_providers.push(rpc);
        }

        let blast_provider = config.blast_provider.clone();
        if blast_provider.is_available(&config.chain) {
            let rpc = Rpc::new(&config, &pokt_provider).await.unwrap();
            available_providers.push(rpc);
        }

        if config.fallback_rpc != String::from("") {
            let provider = &Provider {
                name: String::from("fallback"),
                http: config.fallback_rpc.clone(),
                wss: String::from(""),
                wss_access: false,
            };

            let rpc = Rpc::new(&config, provider).await.unwrap();
            available_providers.push(rpc);
        }
    } else {
        let local_rpc = config.local_rpc.clone();
        let local_rpc_ws = config.local_rpc_ws.clone();

        let provider = &Provider {
            name: String::from("local"),
            http: local_rpc,
            wss: local_rpc_ws,
            wss_access: true,
        };

        let rpc = Rpc::new(&config, provider).await.unwrap();

        available_providers.push(rpc);
    }

    tokio::spawn({
        let db = db.clone();
        let config = config.clone();
        let available_providers = available_providers.clone();

        async move {
            loop {
                fetcher::fetch_blocks(&available_providers, &db, &config)
                    .await
                    .unwrap();
                sleep(Duration::from_secs(120)).await;
            }
        }
    });

    tokio::spawn({
        let rpc = available_providers[0].clone();
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

    tokio::spawn({
        let db = db.clone();
        let config = config.clone();
        let provider = &Provider {
            name: String::from("fallback"),
            http: config.fallback_rpc.clone(),
            wss: String::from(""),
            wss_access: false,
        };

        let rpc = Rpc::new(&config, provider).await.unwrap();

        async move {
            loop {
                fetcher::fetch_tx_no_receipts(&rpc, &config, &db)
                    .await
                    .unwrap();
            }
        }
    });

    loop {
        let rpc = available_providers[0].clone();
        rpc.subscribe_heads(&db).await;
        sleep(Duration::from_secs(180)).await;
    }
}
