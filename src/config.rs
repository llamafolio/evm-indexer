pub const DEFAULT_FETCHER_BATCH_SIZE: usize = 200;
pub const DEFAULT_AMOUNT_OF_WORKERS: usize = 10;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub rpc_http_url: String,
    pub rpc_ws_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            rpc_http_url: std::env::var("RPC_HTTP_URL").expect("RPC_HTTP_URL must be set."),
            rpc_ws_url: std::env::var("RPC_WS_URL").expect("RPC_WS_URL must be set."),
        }
    }
}
