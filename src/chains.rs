pub const AVAILABLE_CHAINS: &'static [&'static str] = &["mainnet", "bsc"];

pub const AVAILABLE_PROVIDERS: &'static [&'static str] = &["ankr"];

pub struct Endpoints {
    pub http: String,
    pub wss: String,
}

pub fn get_endpoints(provider: String, chain: String, key: String) -> Endpoints {
    if provider == "ankr" {
        return get_ankr_endpoint(chain, key);
    }

    return Endpoints {
        http: String::from(""),
        wss: String::from(""),
    };
}

fn get_ankr_endpoint(chain: String, key: String) -> Endpoints {
    let mut slug = chain.clone();

    if chain == String::from("mainnet") {
        slug = String::from("eth");
    }

    return Endpoints {
        http: format!("https://rpc.ankr.com/{}/{}", slug, key),
        wss: format!("wss://rpc.ankr.com/{}/ws/{}", slug, key),
    };
}
