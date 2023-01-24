use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "EVM Parser",
    about = "Transaction and Logs parser for the EVM indexer."
)]
pub struct EVMParserArgs {
    #[arg(short, long, help = "Start log with debug", default_value_t = false)]
    pub debug: bool,

    #[arg(
        short,
        long,
        help = "Start the llamafolio adapters fetcher",
        default_value_t = false
    )]
    pub llamafolio_adapters: bool,

    #[arg(
        short,
        long,
        help = "Start the erc20 tokens parser",
        default_value_t = false
    )]
    pub erc20_tokens_parser: bool,

    #[arg(
        short,
        long,
        help = "Start the erc20 balances parser",
        default_value_t = false
    )]
    pub erc20_balances_parser: bool,
}

#[derive(Debug, Clone)]
pub struct EVMParserConfig {
    pub db_url: String,
    pub redis_url: String,
    pub debug: bool,
    pub llamafolio_adapter: bool,
    pub erc20_tokens_parser: bool,
    pub erc20_balances_parser: bool,
}

impl EVMParserConfig {
    pub fn new() -> Self {
        let args = EVMParserArgs::parse();

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            redis_url: std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
            debug: args.debug,
            llamafolio_adapter: args.llamafolio_adapters,
            erc20_tokens_parser: args.erc20_tokens_parser,
            erc20_balances_parser: args.erc20_balances_parser,
        }
    }
}
