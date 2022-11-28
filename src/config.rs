use clap::Parser;

use crate::chains::{get_chain, Chain};

pub const DEFAULT_FETCHER_BATCH_SIZE: usize = 100;
pub const DEFAULT_AMOUNT_OF_WORKERS: usize = 20;

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

    #[arg(short, long, help = "RPC provider to use to sync", default_value_t = String::from("llamanodes"))]
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
    pub provider: String,
    pub chain: Chain,
}

impl Config {
    pub fn new() -> Self {
        let args = Args::parse();

        let mut chainname = args.chain;

        if chainname == "ethereum" {
            chainname = "mainnet".to_string();
        }

        let chain = get_chain(chainname);

        let provider_key = std::env::var("PROVIDER_KEY").expect("PROVIDER_KEY must be set.");

        let provider = args.provider;

        let endpoints = chain.get_endpoints(provider_key, provider.clone());

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            rpc_http_url: endpoints.http,
            rpc_ws_url: endpoints.wss,
            debug: args.debug,
            start_block: args.start_block,
            workers: args.workers,
            batch_size: args.batch_size,
            provider,
            chain,
        }
    }
}
