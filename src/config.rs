use clap::Parser;

pub const DEFAULT_FETCHER_BATCH_SIZE: usize = 500;
pub const DEFAULT_AMOUNT_OF_WORKERS: usize = 10;

#[derive(Parser, Debug)]
#[command(
    name = "EVM Indexer",
    about = "Minimalistc EVM chain compatible indexer."
)]
pub struct Args {
    #[arg(short, long, help = "Start log with debug", default_value_t = false)]
    pub debug: bool,
    #[arg(
        short, long,
        help = "Amount of workers to fetch blocks",
        default_value_t = DEFAULT_AMOUNT_OF_WORKERS
    )]
    pub workers: usize,
    #[arg(short, long, help = "Initial block to fetch from", default_value_t = 0)]
    pub initial_block: usize,
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
    pub initial_block: usize,
    pub workers: usize,
    pub batch_size: usize,
}

impl Config {
    pub fn new() -> Self {
        let args = Args::parse();

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            rpc_http_url: std::env::var("RPC_HTTP_URL").expect("RPC_HTTP_URL must be set."),
            rpc_ws_url: std::env::var("RPC_WS_URL").expect("RPC_WS_URL must be set."),
            debug: args.debug,
            initial_block: args.initial_block,
            workers: args.workers,
            batch_size: args.batch_size,
        }
    }
}
