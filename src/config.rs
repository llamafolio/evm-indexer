use clap::Parser;

use crate::chains::{get_chain, Chain, Provider};

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
    pub debug: bool,
    pub start_block: i64,
    pub workers: usize,
    pub batch_size: usize,
    pub chain: Chain,
    pub llamanodes_provider: Provider,
    pub ankr_provider: Provider,
    pub pokt_provider: Provider,
    pub blast_provider: Provider,
    pub fall_back_rpc: String,
}

impl Config {
    pub fn new() -> Self {
        let args = Args::parse();

        let mut chainname = args.chain;

        if chainname == "ethereum" {
            chainname = "mainnet".to_string();
        }

        let chain = get_chain(chainname);

        let llamanodes_key = match std::env::var("LLAMANODES_PROVIDER_ID") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };

        let ankr_key = match std::env::var("ANKR_PROVIDER_ID") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };

        let pokt_key = match std::env::var("POKT_PROVIDER_ID") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };

        let blast_key = match std::env::var("BLAST_PROVIDER_ID") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };

        let llamanodes_provider = chain.get_provider(llamanodes_key, "llamanodes".to_string());

        let ankr_provider = chain.get_provider(ankr_key, "ankr".to_string());

        let pokt_provider = chain.get_provider(pokt_key, "pokt".to_string());

        let blast_provider = chain.get_provider(blast_key, "blast".to_string());

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            debug: args.debug,
            start_block: args.start_block,
            workers: args.workers,
            batch_size: args.batch_size,
            chain,
            llamanodes_provider,
            ankr_provider,
            pokt_provider,
            blast_provider,
            fall_back_rpc: get_fallback_rpc(chain.name.to_string()),
        }
    }
}

pub fn get_fallback_rpc(chain: String) -> String {
    let mut fallback_rpc = String::from("");

    if chain == "mainnet" {
        fallback_rpc = match std::env::var("ETH_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "bsc" {
        fallback_rpc = match std::env::var("BSC_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "gnosis" {
        fallback_rpc = match std::env::var("GNOSIS_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "avalanche" {
        fallback_rpc = match std::env::var("AVAX_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "fantom" {
        fallback_rpc = match std::env::var("FTM_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "polygon" {
        fallback_rpc = match std::env::var("POLYGON_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    if chain == "optimism" {
        fallback_rpc = match std::env::var("OPTIMISM_FALLBACK_RPC") {
            Ok(key) => key,
            Err(_) => String::from(""),
        };
    }

    return fallback_rpc;
}
