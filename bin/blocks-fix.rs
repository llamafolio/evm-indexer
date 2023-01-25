use diesel::{ prelude::*, connection::SimpleConnection };
use dotenv::dotenv;
use evm_indexer::{
    chains::chains::ETHEREUM,
    db::{ db::Database, schema::blocks },
    utils::format_bytes_slice,
};

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

        let mut count = 0;

        let mut query = String::new();

        for (block_hash, logs_bloom) in blocks {
            let bloom_vec: Vec<u8> = match serde_json::from_str(&logs_bloom) {
                Ok(data) => data,
                Err(_) => {
                    diesel
                        ::update(blocks::dsl::blocks)
                        .filter(blocks::block_hash.eq(block_hash))
                        .set(blocks::parsed.eq(true))
                        .execute(&mut connection)
                        .unwrap();

                    continue;
                }
            };

            let formatted = format_bytes_slice(&bloom_vec[..]);

            let sql = format!(
                "UPDATE blocks SET logs_bloom = '{}',parsed = true WHERE block_hash = '{}';",
                formatted,
                block_hash
            );

            query.push_str(&sql);

            count += 1;
        }

        if query != String::from("") {
            connection.batch_execute(&query).unwrap();
        }

        println!("Fixed {} blocks", count);
    }
}