use diesel::prelude::*;
use dotenv::dotenv;
use evm_indexer::{
    chains::chains::ETHEREUM,
    db::{db::EVMDatabase, schema::blocks},
    utils::format_bytes_slice,
};
use log::*;

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let db = EVMDatabase::new(
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
        std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
        ETHEREUM,
    )
    .await
    .unwrap();

    info!("Fixing blocks log_blooms");

    let mut connection = db.establish_connection();

    let blocks: Vec<(String, String)> = blocks::dsl::blocks
        .select((blocks::block_hash, blocks::logs_bloom))
        .load::<(String, String)>(&mut connection)
        .unwrap();

    for (block_hash, logs_bloom) in blocks {
        let bloom_vec: Vec<u8> = match serde_json::from_str(&logs_bloom) {
            Ok(data) => data,
            Err(_) => continue,
        };

        let formatted = format_bytes_slice(&bloom_vec[..]);

        diesel::update(blocks::dsl::blocks)
            .filter(blocks::block_hash.eq(block_hash))
            .set(blocks::logs_bloom.eq(formatted))
            .execute(&mut connection)
            .unwrap();
    }
}
