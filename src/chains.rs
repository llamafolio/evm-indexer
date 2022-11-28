use std::collections::HashMap;

pub struct Endpoints {
    pub http: String,
    pub wss: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub multicall_address: &'static str,
}

impl Chain {
    pub fn new_from_borrowed(chain: &Chain) -> Self {
        Self {
            id: chain.id,
            name: chain.name,
            multicall_address: chain.multicall_address,
        }
    }

    pub fn get_endpoints(&self, key: String, provider: String) -> Endpoints {
        let name = self.name;

        let mut slug = name;

        if name == String::from("mainnet") {
            slug = "eth"
        }

        if name == String::from("fantom") {
            slug = "ftm"
        }

        if provider == "llamanodes" {
            return Endpoints {
                http: format!("https://{}-ski.llamarpc.com/rpc/{}", slug, key),
                wss: format!("wss://{}-ski.llamarpc.com/rpc/{}", slug, key),
            };
        } else if provider == "ankr" {
            return Endpoints {
                http: format!("https://rpc.ankr.com/{}/{}", slug, key),
                wss: format!("wss://rpc.ankr.com/{}/ws/{}", slug, key),
            };
        } else {
            return Endpoints {
                http: String::from(""),
                wss: String::from(""),
            };
        }
    }
}

static ETHEREUM: Chain = Chain {
    id: 1,
    name: "mainnet",
    multicall_address: "0xcA11bde05977b3631167028862bE2a173976CA11",
};

static POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    multicall_address: "0xcA11bde05977b3631167028862bE2a173976CA11",
};

static FTM: Chain = Chain {
    id: 250,
    name: "fantom",
    multicall_address: "0xcA11bde05977b3631167028862bE2a173976CA11s",
};

pub static AVAILABLE_CHAINS: [Chain; 3] = [ETHEREUM, POLYGON, FTM];

pub fn get_chains() -> HashMap<String, Chain> {
    let mut chains: HashMap<String, Chain> = HashMap::new();

    for chain in AVAILABLE_CHAINS.into_iter() {
        chains.insert(String::from(chain.name), chain);
    }

    return chains;
}

pub fn get_chain(chain: String) -> Chain {
    let chains = get_chains();

    let selected_chain = chains.get(&chain).expect("Invalid chain name");

    return Chain::new_from_borrowed(selected_chain);
}
