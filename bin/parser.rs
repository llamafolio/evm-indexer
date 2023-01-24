use std::{thread::sleep, time::Duration};

use dotenv::dotenv;
use evm_indexer::{
    chains::chains::ETHEREUM,
    configs::parser_config::EVMParserConfig,
    db::db::EVMDatabase,
    parsers::{
        erc20_balances_parser::ERC20BalancesParser, erc20_tokens_parser::ERC20TokensParser,
        erc20_transfers_parser::ERC20TransfersParser, llamafolio_adapters::LlamafolioParser,
    },
};
use log::*;
use simple_logger::SimpleLogger;

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let config = EVMParserConfig::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    info!("Starting EVM Parser.");

    let db = EVMDatabase::new(config.db_url, config.redis_url.clone(), ETHEREUM)
        .await
        .expect("Unable to start DB connection.");

    if config.llamafolio_adapter {
        info!("Starting the LlamaFolio adapters fetcher.");

        tokio::spawn({
            let db = db.clone();
            async move {
                loop {
                    let llamafolio_adapters = LlamafolioParser {};

                    let adapters = llamafolio_adapters.fetch().await.unwrap();

                    info!("Fetched {} adapters.", adapters.len());

                    llamafolio_adapters.parse(&db, &adapters).await.unwrap();

                    sleep(Duration::from_secs(1800))
                }
            }
        });
    }

    if config.erc20_tokens_parser {
        info!("Starting the ERC20 Tokens parser.");

        tokio::spawn({
            let db = db.clone();
            async move {
                loop {
                    let parser = ERC20TokensParser {};

                    let data = parser.fetch(&db).unwrap();

                    info!("Fetched {} transfers to parse.", data.len());

                    parser.parse(&db, &data).await.unwrap();

                    sleep(Duration::from_secs(2))
                }
            }
        });
    }

    if config.erc20_balances_parser {
        info!("Starting the ERC20 Balances parser.");

        tokio::spawn({
            let db = db.clone();
            async move {
                loop {
                    let parser = ERC20BalancesParser {};

                    let data = parser.fetch(&db).unwrap();

                    info!("Fetched {} transfers to parse.", data.len());

                    parser.parse(&db, &data).await.unwrap();

                    sleep(Duration::from_secs(2))
                }
            }
        });
    }

    info!("Starting the ERC20 Transfers parser.");

    loop {
        let erc20_transfers_parser = ERC20TransfersParser {};

        let logs = erc20_transfers_parser.fetch(&db).unwrap();

        info!("Fetched {} logs to parse.", logs.len());

        erc20_transfers_parser.parse(&db, &logs).await.unwrap();

        sleep(Duration::from_secs(2))
    }
}
