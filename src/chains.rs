use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Provider {
    pub name: String,
    pub http: String,
    pub wss: String,
    pub wss_access: bool,
}

impl Provider {
    pub fn is_available(&self, chain: &Chain) -> bool {
        if self.name == "ankr" {
            if !chain.ankr_available {
                return false;
            }
        }

        if self.name == "llamanodes" {
            if !chain.llamanodes_available {
                return false;
            }
        }

        if self.name == "pokt" {
            if !chain.pokt_available {
                return false;
            }
        }

        return self.http != String::from("");
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub blocks_reorg: i64,
    pub ankr_available: bool,
    pub llamanodes_available: bool,
    pub pokt_available: bool,
    pub blast_available: bool,
}

impl Chain {
    pub fn new_from_borrowed(chain: &Chain) -> Self {
        Self {
            id: chain.id,
            name: chain.name,
            blocks_reorg: chain.blocks_reorg,
            ankr_available: chain.ankr_available,
            llamanodes_available: chain.llamanodes_available,
            pokt_available: chain.pokt_available,
            blast_available: chain.blast_available,
        }
    }

    pub fn get_provider(&self, key: String, provider: String) -> Provider {
        let name = self.name;

        let mut slug = name;

        if key == String::from("") {
            return Provider {
                name: String::from(""),
                http: String::from(""),
                wss: String::from(""),
                wss_access: false,
            };
        }

        if name == String::from("mainnet") {
            slug = "eth"
        }

        // Rules for slug changes for llamanodes
        if name == String::from("fantom") && provider == "llamanodes" {
            slug = "ftm"
        }

        if name == String::from("fantom") && provider == "llamanodes" {
            slug = "ftm"
        }

        // Rules for slug changes for pokt provider
        if name == String::from("avalanche") && provider == "pokt" {
            slug = "avax"
        }

        if name == String::from("polygon") && provider == "pokt" {
            slug = "poly"
        }

        // Rules for slug changes for blast provider
        if name == String::from("avalanche") && provider == "blast" {
            slug = "ava"
        }

        if provider == "llamanodes" {
            return Provider {
                name: "llamanodes".to_string(),
                http: format!("https://{}.llamarpc.com:443/rpc/{}", slug, key),
                wss: format!("wss://{}.llamarpc.com:443/rpc/{}", slug, key),
                wss_access: true,
            };
        } else if provider == "ankr" {
            return Provider {
                name: "ankr".to_string(),
                http: format!("https://rpc.ankr.com:443/{}/{}", slug, key),
                wss: format!("wss://rpc.ankr.com:443/{}/ws/{}", slug, key),
                wss_access: true,
            };
        } else if provider == "pokt" {
            let mut net = "mainnet";

            if name == String::from("gnosis") {
                net = "xdai-archival"
            }

            if name == String::from("mainnet")
                || name == String::from("poly")
                || name == String::from("bsc")
            {
                net = "archival"
            }

            return Provider {
                name: "pokt".to_string(),
                http: format!(
                    "https://{}-{}.gateway.pokt.network:443/v1/lb/{}",
                    slug, net, key
                ),
                wss: String::from(""),
                wss_access: false,
            };
        } else if provider == "blast" {
            return Provider {
                name: "blast".to_string(),
                http: format!("https://{}-mainnet.blastapi.io:443/{}", slug, key),
                wss: format!("wss://{}-mainnet.blastapi.io:443/{}", slug, key),
                wss_access: false,
            };
        } else {
            return Provider {
                name: String::from(""),
                http: String::from(""),
                wss: String::from(""),
                wss_access: false,
            };
        }
    }
}

static ETHEREUM: Chain = Chain {
    id: 1,
    name: "mainnet",
    blocks_reorg: 12,
    ankr_available: true,
    llamanodes_available: true,
    pokt_available: true,
    blast_available: true,
};

static POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    blocks_reorg: 128,
    ankr_available: true,
    llamanodes_available: true,
    pokt_available: true,
    blast_available: true,
};

static FTM: Chain = Chain {
    id: 250,
    name: "fantom",
    blocks_reorg: 5,
    ankr_available: true,
    llamanodes_available: true,
    pokt_available: true,
    blast_available: true,
};

static OPTIMISM: Chain = Chain {
    id: 10,
    name: "optimism",
    blocks_reorg: 20,
    ankr_available: true,
    llamanodes_available: false,
    pokt_available: true,
    blast_available: true,
};

static ARBITTUM: Chain = Chain {
    id: 42161,
    name: "arbitrum",
    blocks_reorg: 20,
    ankr_available: true,
    llamanodes_available: false,
    pokt_available: true,
    blast_available: true,
};

static GNOSIS: Chain = Chain {
    id: 20,
    name: "gnosis",
    blocks_reorg: 20,
    ankr_available: true,
    llamanodes_available: false,
    pokt_available: true,
    blast_available: true,
};

static BNB_CHAIN: Chain = Chain {
    id: 56,
    name: "bsc",
    blocks_reorg: 16,
    ankr_available: true,
    llamanodes_available: false,
    pokt_available: true,
    blast_available: true,
};

static AVALANCHE: Chain = Chain {
    id: 43114,
    name: "avalanche",
    blocks_reorg: 16,
    ankr_available: true,
    llamanodes_available: false,
    pokt_available: true,
    blast_available: true,
};

pub static AVAILABLE_CHAINS: [Chain; 8] = [
    ETHEREUM, POLYGON, FTM, OPTIMISM, ARBITTUM, GNOSIS, BNB_CHAIN, AVALANCHE,
];

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
