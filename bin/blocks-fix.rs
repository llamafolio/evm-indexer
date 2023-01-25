use diesel::prelude::*;
use dotenv::dotenv;
use evm_indexer::{
    chains::chains::ETHEREUM,
    db::{ db::Database, schema::blocks },
    utils::format_bytes_slice,
};
use futures::future::join_all;

async fn fix_block(db: &Database, hash: String, bloom: String) {
    let bloom_vec: Vec<u8> = match serde_json::from_str(&bloom) {
        Ok(data) => data,
        Err(_) => {
            return;
        }
    };

    let formatted = format_bytes_slice(&bloom_vec[..]);

    let mut connection = db.establish_connection();

    diesel
        ::update(blocks::dsl::blocks)
        .filter(blocks::block_hash.eq(hash))
        .set((blocks::logs_bloom.eq(formatted), blocks::parsed.eq(true)))
        .execute(&mut connection)
        .unwrap();
}

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let db = Database::new(
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
        std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
        ETHEREUM
    ).await.unwrap();

    println!("Fixing blocks log_blooms");

    let mut connection = db.establish_connection();

    loop {
        let blocks: Vec<(String, String)> = blocks::dsl::blocks
            .select((blocks::block_hash, blocks::logs_bloom))
            .filter(blocks::parsed.eq(false))
            .limit(1000)
            .load::<(String, String)>(&mut connection)
            .unwrap();

        println!("Fetched {} blocks to fix", blocks.len());

        let mut works = vec![];

        for (block_hash, logs_bloom) in blocks {
            works.push(fix_block(&db, block_hash, logs_bloom));
        }

        join_all(works).await;
    }
}