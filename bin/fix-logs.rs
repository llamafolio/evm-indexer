use diesel::sql_query;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use evm_indexer::chains::chains::ETHEREUM;
use evm_indexer::configs::abi_fetcher_config::EVMAbiFetcherConfig;
use evm_indexer::db::db::Database;
use log::LevelFilter;
use log::*;
use simple_logger::SimpleLogger;

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let config = EVMAbiFetcherConfig::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    info!("Starting EVM ABI fetcher");

    let db = Database::new(config.db_url, config.redis_url.clone(), ETHEREUM)
        .await
        .expect("Unable to start DB connection.");

    let times = 0..3070;

    let mut connection = db.establish_connection();

    sql_query("SET experimental_enable_temp_tables=on;")
        .execute(&mut connection)
        .unwrap();

    for i in times {
        println!("Running fix script time {}", i);

        sql_query("CREATE TEMP TABLE logs_modify AS SELECT * FROM logs WHERE logs.chain = 'nochain' LIMIT 1000000;").execute(&mut connection).unwrap();
        sql_query("CREATE INDEX ON logs_modify(chain);")
            .execute(&mut connection)
            .unwrap();
        sql_query(
            " UPDATE logs l SET chain = lm.chain FROM logs_modify lm WHERE l.hash = lm.hash;",
        )
        .execute(&mut connection)
        .unwrap();
        sql_query("DROP TABLE logs_modify;")
            .execute(&mut connection)
            .unwrap();
    }
}
