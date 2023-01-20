use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "EVM Parser",
    about = "Transaction and Logs parser for the EVM indexer."
)]
pub struct EVMParserArgs {
    #[arg(short, long, help = "Start log with debug", default_value_t = false)]
    pub debug: bool,
}

#[derive(Debug, Clone)]
pub struct EVMParserConfig {
    pub db_url: String,
    pub redis_url: String,
    pub debug: bool,
}

impl EVMParserConfig {
    pub fn new() -> Self {
        let args = EVMParserArgs::parse();

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            redis_url: std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
            debug: args.debug,
        }
    }
}
