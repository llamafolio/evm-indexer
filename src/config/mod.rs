const DEFAULT_FETCHER_BATCH_SIZE: usize = 200;
const DEFAULT_AMOUNT_OF_WORKERS: usize = 10;

pub struct IndexerConfig {
    pub db_url: String,
    pub db_name: String,
    pub rpc_http_url: String,
    pub rpc_ws_url: String,
    pub initial_block: usize,
    pub workers: usize,
    pub batch_size: usize,
    pub listen_addr: String,
    pub listen_port: String,
}

impl IndexerConfig {
    pub fn new() -> Self {
        let initial_sync_variable = std::env::var("INITIAL_SYNC_START_BLOCK");

        let mut initial_block: usize = 0;

        match initial_sync_variable {
            Ok(v) => {
                initial_block = v
                    .parse()
                    .expect("INITIAL_SYNC_START_BLOCK must be a number");
            }
            Err(_) => (),
        };

        let workers_variable = std::env::var("WORKERS_AMOUNT");

        let mut workers: usize = DEFAULT_AMOUNT_OF_WORKERS;

        match workers_variable {
            Ok(v) => {
                workers = v.parse().expect("WORKERS_AMOUNT must be a number");
            }
            Err(_) => (),
        };

        let batch_size_variable = std::env::var("BLOCK_PARSE_BATCH_SIZE");

        let mut batch_size: usize = DEFAULT_FETCHER_BATCH_SIZE;

        match batch_size_variable {
            Ok(v) => batch_size = v.parse().expect("BLOCK_PARSE_BATCH_SIZE must be a number"),
            Err(_) => (),
        };

        IndexerConfig {
            db_url: std::env::var("DB_URL").expect("DB_URL must be set."),
            db_name: std::env::var("DB_NAME").expect("DB_NAME must be set."),
            rpc_http_url: std::env::var("RPC_HTTPS_URL").expect("RPC_HTTPS_URL must be set."),
            rpc_ws_url: std::env::var("RPC_WS_URL").expect("RPC_WS_URL must be set."),
            initial_block,
            batch_size,
            workers,
            listen_addr: String::from("0.0.0.0"),
            listen_port: String::from("9000"),
        }
    }
}
