use std::collections::HashMap;

pub struct Endpoints {
    pub http: String,
    pub wss: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub blocks_reorg: i64,
}

impl Chain {
    pub fn new_from_borrowed(chain: &Chain) -> Self {
        Self {
            id: chain.id,
            name: chain.name,
            blocks_reorg: chain.blocks_reorg,
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
    blocks_reorg: 12,
};

static POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    blocks_reorg: 128,
};

static FTM: Chain = Chain {
    id: 250,
    name: "fantom",
    blocks_reorg: 5,
};

static OPTIMISM: Chain = Chain {
    id: 10,
    name: "optimism",
    blocks_reorg: 20,
};

static ARBITTUM: Chain = Chain {
    id: 42161,
    name: "arbitrum",
    blocks_reorg: 20,
};

pub static AVAILABLE_CHAINS: [Chain; 5] = [ETHEREUM, POLYGON, FTM, OPTIMISM, ARBITTUM];

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
