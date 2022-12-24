use std::time::Duration;

use dotenv::dotenv;
use evm_indexer::{config::Config, db::Database, fetcher, rpc::Rpc};
use log::*;
use simple_logger::SimpleLogger;
use tokio::time::sleep;

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

    let rpc = Rpc::new(&config)
        .await
        .expect("Unable to connect to the rpc");

    tokio::spawn({
        let db = db.clone();
        let config = config.clone();
        let rpc = rpc.clone();

        async move {
            loop {
                match fetcher::fetch_blocks(&db, &config, &rpc).await {
                    Ok(_) => {
                        sleep(Duration::from_secs(5)).await;
                    }
                    Err(err) => println!("{}", err),
                };
            }
        }
    });

    tokio::spawn({
        let db = db.clone();
        let config = config.clone();
        let rpc = rpc.clone();

        async move {
            loop {
                match fetcher::fetch_tokens_metadata(&rpc, &db, &config).await {
                    Ok(_) => {
                        sleep(Duration::from_secs(5)).await;
                    }
                    Err(err) => println!("{}", err),
                };
            }
        }
    });

    tokio::spawn({
        let db = db.clone();
        let config = config.clone();
        let rpc = rpc.clone();

        async move {
            loop {
                match fetcher::fetch_tx_no_receipts(&rpc, &config, &db).await {
                    Ok(_) => {
                        sleep(Duration::from_secs(5)).await;
                    }
                    Err(err) => println!("{}", err),
                };
            }
        }
    });

    let abi_source_token = config.abi_source_api_token.clone();

    if abi_source_token != String::from("") {
        tokio::spawn({
            let db = db.clone();
            let config = config.clone();
            let abi_source_token = abi_source_token.clone();

            async move {
                loop {
                    match fetcher::fetch_contract_abis(&config, &db, &abi_source_token).await {
                        Ok(_) => {
                            sleep(Duration::from_secs(5)).await;
                        }
                        Err(err) => println!("{}", err),
                    };
                }
            }
        });
    }

    tokio::spawn({
        let db = db.clone();
        let config = config.clone();

        async move {
            loop {
                match fetcher::fetch_adapters(&config, &db).await {
                    Ok(_) => {
                        sleep(Duration::from_secs(1800)).await;
                    }
                    Err(err) => println!("{}", err),
                };
            }
        }
    });

    tokio::spawn({
        let db = db.clone();
        async move {
            loop {
                db.update_chain_state().await.unwrap();
                sleep(Duration::from_secs(1800)).await;
            }
        }
    });

    loop {
        let rpc = rpc.clone();
        rpc.subscribe_heads(&config, &db).await;
        sleep(Duration::from_secs(5)).await;
    }
}
