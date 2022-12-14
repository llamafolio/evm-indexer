use clap::Parser;

use crate::chains::{get_chain, Chain};

pub const DEFAULT_FETCHER_BATCH_SIZE: usize = 100;

pub const DEFAULT_FETCHER_WORKERS: usize = 20;

pub const DEFAULT_RECEIPTS_FETCH_AMOUNT: usize = 10000;

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

    #[arg(
        short, long,
        help = "Amount of blocks to fetch by batch",
        default_value_t = DEFAULT_FETCHER_BATCH_SIZE
    )]
    pub batch_size: usize,

    #[arg(
        short, long,
        help = "Amount of receipts to fetch by batch",
        default_value_t = DEFAULT_RECEIPTS_FETCH_AMOUNT
    )]
    pub receipts_batch_size: usize,

    #[arg(
        short, long,
        help = "Amount of workers to fetch blocks",
        default_value_t = DEFAULT_FETCHER_WORKERS
    )]
    pub workers: usize,

    #[arg(
        short,
        long,
        help = "Port of the RPC local node",
        default_value_t = String::from("8545")
    )]
    pub rpc_port: String,

    #[arg(
        long,
        help = "Fetch the blocks in singles instead of batches",
        default_value_t = false
    )]
    pub singles_fetch: bool,

    #[arg(
        short,
        long,
        help = "Port of the WSS local node",
        default_value_t = String::from("8545")
    )]
    pub wss_port: String,

    #[arg(
        long,
        help = "URL of the remote server to use if not syncing from local",
        default_value_t = String::from("")
    )]
    pub remote_rpc: String,

    #[arg(
        long,
        help = "WSS of the remote server to use if not syncing from local",
        default_value_t = String::from("")
    )]
    pub remote_wss: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub debug: bool,
    pub batch_size: usize,
    pub receipts_batch_size: usize,
    pub fetch_in_singles: bool,
    pub workers: usize,
    pub chain: Chain,
    pub abi_source_api_token: String,
    pub local_rpc_http: String,
    pub local_rpc_wss: String,
    pub remote_rpc: String,
    pub remote_wss: String,
}

impl Config {
    pub fn new() -> Self {
        let args = Args::parse();

        let mut chainname = args.chain;

        if chainname == "ethereum" {
            chainname = "mainnet".to_string();
        }

        let chain = get_chain(chainname.clone());

        let abi_source_api_token = get_abi_source_token(chainname.clone());

        let mut local_rpc_wss: String = format!("ws://localhost:{}", args.wss_port);

        if chainname == "dogechain" {
            local_rpc_wss = format!("ws://localhost:{}/ws", args.rpc_port);
        }

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            debug: args.debug,
            batch_size: args.batch_size,
            receipts_batch_size: args.receipts_batch_size,
            workers: args.workers,
            chain,
            abi_source_api_token,
            local_rpc_http: format!("http://localhost:{}", args.rpc_port),
            local_rpc_wss,
            remote_rpc: args.remote_rpc,
            remote_wss: args.remote_wss,
            fetch_in_singles: args.singles_fetch,
        }
    }
}

pub fn get_abi_source_token(chain: String) -> String {
    let mut abi_source_token = String::from("");

    if chain == "mainnet" {
        abi_source_token = match std::env::var("ETHERSCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "bsc" {
        abi_source_token = match std::env::var("BSCSCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "gnosis" {
        abi_source_token = match std::env::var("GNOSISSCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "avalanche" {
        abi_source_token = match std::env::var("SNOWTRACE_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "fantom" {
        abi_source_token = match std::env::var("FTMSCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "polygon" {
        abi_source_token = match std::env::var("POLYGONSCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "optimism" {
        abi_source_token = match std::env::var("OPTIMISMSCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "arbitrum" {
        abi_source_token = match std::env::var("ARBISCAN_API_TOKEN") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    return abi_source_token;
}
