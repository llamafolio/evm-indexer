use dotenv::dotenv;
use mongodb::{ Client, options::ClientOptions };

fn main() {
    dotenv().ok();

    // Load .env variables
    let mongodb_url = std::env::var("DB_URL").expect("DB_URL must be set.");
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set.");

    // Initialize Web3 and DB services
    let transport = web3::transports::Http::new(&rpc_url);
    //let web3 = web3::Web3::new(transport);

    let client = Client::with_uri_str(&mongodb_url);
}