use evm_indexer::{chains::chains::get_chains, db::db::INDEXED_BLOCKS_DB};
use redis::Commands;
use rocksdb::DB;

#[tokio::main()]
async fn main() {
    let redis =
        redis::Client::open("redis://localhost:6379").expect("Unable to connect with Redis server");

    let indexed_blocks_db = DB::open_default(INDEXED_BLOCKS_DB).unwrap();

    let mut connection = redis.get_connection().unwrap();

    for (chain, _) in get_chains() {
        let raw_data = connection.get::<String, String>(chain.clone()).unwrap();

        let serialized = raw_data.as_bytes();

        indexed_blocks_db.put(chain, serialized).unwrap();
    }
}
