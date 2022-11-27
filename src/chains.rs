pub const AVAILABLE_CHAINS: &'static [&'static str] = &["mainnet", "polygon", "fantom"];

pub const AVAILABLE_PROVIDERS: &'static [&'static str] = &["llamanodes"];

pub struct Endpoints {
    pub http: String,
    pub wss: String,
}

pub fn get_endpoints(provider: String, chain: String, key: String) -> Endpoints {
    if provider == "llamanodes" {
        return get_llamanodes_endpoint(chain, key);
    }

    return Endpoints {
        http: String::from(""),
        wss: String::from(""),
    };
}

fn get_llamanodes_endpoint(chain: String, key: String) -> Endpoints {
    let mut slug = chain.clone();

    if chain == String::from("mainnet") {
        slug = String::from("eth");
    }

    if chain == String::from("fantom") {
        slug = String::from("ftm");
    }

    return Endpoints {
        http: format!("https://{}-ski.llamarpc.com/rpc/{}", slug, key),
        wss: format!("wss://{}-ski.llamarpc.com/rpc/{}", slug, key),
    };
}
