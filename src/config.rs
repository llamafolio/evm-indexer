use clap::Parser;

use crate::chains::{get_endpoints, Endpoints, AVAILABLE_CHAINS, AVAILABLE_PROVIDERS};

pub const DEFAULT_FETCHER_BATCH_SIZE: usize = 200;
pub const DEFAULT_AMOUNT_OF_WORKERS: usize = 10;

#[derive(Parser, Debug)]
#[command(
    name = "EVM Indexer",
    about = "Minimalistc EVM chain compatible indexer."
)]
pub struct Args {
    #[arg(short, long, help = "Start log with debug", default_value_t = false)]
    pub debug: bool,

    #[arg(short, long, help = "Chain name to sync", default_value_t = String::from("mainnet"))]
    pub chain: String,

    #[arg(short, long, help = "Name of the provider used to sync", default_value_t = String::from("ankr"))]
    pub provider: String,

    #[arg(
        short, long,
        help = "Amount of workers to fetch blocks",
        default_value_t = DEFAULT_AMOUNT_OF_WORKERS
    )]
    pub workers: usize,

    #[arg(short, long, help = "Initial block to fetch from", default_value_t = 1)]
    pub start_block: i64,

    #[arg(
        short, long,
        help = "Amount of blocks to fetch by batch",
        default_value_t = DEFAULT_FETCHER_BATCH_SIZE
    )]
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub rpc_http_url: String,
    pub rpc_ws_url: String,
    pub debug: bool,
    pub start_block: i64,
    pub workers: usize,
    pub batch_size: usize,
    pub chain: String,
}

impl Config {
    pub fn new() -> Self {
        let args = Args::parse();

        let mut chain = args.chain;

        if chain == "ethereum" {
            chain = "mainnet".to_string();
        }

        if !AVAILABLE_CHAINS.contains(&&*chain.clone()) {
            panic!("Chain not available");
        }

        let provider = String::from(args.provider);

        if !AVAILABLE_PROVIDERS.contains(&&*provider) {
            panic!("Provider not available");
        }

        let provider_key = std::env::var("PROVIDER_KEY").expect("PROVIDER_KEY must be set.");

        let enpoints: Endpoints = get_endpoints(provider, chain.clone(), provider_key);

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            rpc_http_url: enpoints.http,
            rpc_ws_url: enpoints.wss,
            debug: args.debug,
            start_block: args.start_block,
            workers: args.workers,
            batch_size: args.batch_size,
            chain,
        }
    }
}
