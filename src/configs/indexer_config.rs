use crate::chains::chains::{get_chain, Chain};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "EVM Indexer",
    about = "A scalable SQL indexer for EVM compatible blockchains."
)]
pub struct EVMIndexerArgs {
    #[arg(long, help = "Start log with debug.", default_value_t = false)]
    pub debug: bool,

    #[arg(long, help = "Chain name to sync.", default_value_t = String::from("mainnet"))]
    pub chain: String,

    #[arg(long, help = "Block to start syncing.", default_value_t = 0)]
    pub start_block: i64,

    #[arg(
        long,
        help = "Amount of blocks to fetch at the same time.",
        default_value_t = 200
    )]
    pub batch_size: usize,

    #[arg(
        long,
        help = "Reset a chain to restart the index.",
        default_value_t = false
    )]
    pub reset: bool,

    #[arg(
        long,
        help = "Recalculate the amount of blocks indexed.",
        default_value_t = false
    )]
    pub recalculate_indexed_blocks: bool,

    #[arg(long, help = "Websocket to fetch blocks from.")]
    pub websocket: String,

    #[arg(long, help = "Comma separated list of rpcs to use to fetch blocks.")]
    pub rpcs: String,
}

#[derive(Debug, Clone)]
pub struct EVMIndexerConfig {
    pub start_block: i64,
    pub db_url: String,
    pub redis_url: String,
    pub debug: bool,
    pub chain: Chain,
    pub batch_size: usize,
    pub reset: bool,
    pub websocket: String,
    pub rpcs: Vec<String>,
    pub recalc_blocks_indexer: bool,
}

impl EVMIndexerConfig {
    pub fn new() -> Self {
        let args = EVMIndexerArgs::parse();

        let mut chainname = args.chain;

        if chainname == "mainnet" {
            chainname = "ethereum".to_string();
        }

        let chain = get_chain(chainname.clone());

        let rpcs: Vec<String> = args.rpcs.split(",").map(|rpc| rpc.to_string()).collect();

        Self {
            start_block: args.start_block,
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            redis_url: std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
            debug: args.debug,
            chain,
            batch_size: args.batch_size,
            reset: args.reset,
            websocket: args.websocket,
            rpcs,
            recalc_blocks_indexer: args.recalculate_indexed_blocks,
        }
    }
}
