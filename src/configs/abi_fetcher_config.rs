use std::collections::HashMap;

use clap::Parser;

use crate::chains::chains::get_chains;

#[derive(Parser, Debug)]
#[command(
    name = "EVM ABI Fetcher",
    about = "ABI Fetcher for the EVM indexed chains."
)]
pub struct EVMAbiFetcherArgs {
    #[arg(long, help = "Start log with debug", default_value_t = false)]
    pub debug: bool,
}

#[derive(Debug, Clone)]
pub struct EVMAbiFetcherConfig {
    pub db_url: String,
    pub redis_url: String,
    pub debug: bool,
    pub api_source_tokens: HashMap<String, String>,
}

impl EVMAbiFetcherConfig {
    pub fn new() -> Self {
        let args = EVMAbiFetcherArgs::parse();

        let mut api_source_tokens: HashMap<String, String> = HashMap::new();

        let chains = get_chains();

        for (key, _) in chains {
            match get_abi_token_for_chain(key.clone()) {
                Some(token) => api_source_tokens.insert(key, token),
                None => continue,
            };
        }

        Self {
            db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            redis_url: std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
            debug: args.debug,
            api_source_tokens,
        }
    }
}

pub fn get_abi_token_for_chain(chain: String) -> Option<String> {
    if chain == "ethereum" {
        let token = std::env::var("ETHERSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "polygon" {
        let token = std::env::var("POLYGONSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "bsc" {
        let token = std::env::var("BSCSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "fantom" {
        let token = std::env::var("FTMSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "gnosis" {
        let token = std::env::var("GNOSISSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "optimism" {
        let token = std::env::var("OPTIMISMSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "arbitrum" {
        let token = std::env::var("ARBISCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "arbitrum-nova" {
        let token = std::env::var("ARBISCAN_NOVA_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "moonbeam" {
        let token = std::env::var("MOONSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "avalanche" {
        let token = std::env::var("SNOWTRACE_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "bittorrent" {
        let token = std::env::var("BITTORRENTSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    if chain == "celo" {
        let token = std::env::var("CELOSCAN_TOKEN");
        match token {
            Ok(token) => return Some(token),
            Err(_) => return None,
        }
    }

    return None;
}
