mod db;
mod rpc;

use dotenv::dotenv;

use crate::db::{ IndexerDB };
use crate::rpc::{ IndexerRPC };

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Load .env variables
    let db_url = std::env::var("DB_URL").expect("DB_URL must be set.");
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set.");

    let db = IndexerDB::new(&db_url).await.expect("Unable to connect to the database");
    let rpc = IndexerRPC::new(&rpc_url);

    // Get the last synced block and compare with the RPC
    let last_synced_block = db.last_synced_block().await.unwrap();
}