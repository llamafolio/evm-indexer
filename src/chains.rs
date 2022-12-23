use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub abi_source_url: &'static str,
    pub abi_source_require_auth: bool,
}

impl Chain {
    pub fn new_from_borrowed(chain: &Chain) -> Self {
        Self {
            id: chain.id,
            name: chain.name,
            abi_source_url: chain.abi_source_url,
            abi_source_require_auth: chain.abi_source_require_auth,
        }
    }
}

static ETHEREUM: Chain = Chain {
    id: 1,
    name: "mainnet",
    abi_source_url: "https://api.etherscan.io/",
    abi_source_require_auth: true,
};

static POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    abi_source_url: "https://api.polygonscan.com/",
    abi_source_require_auth: true,
};

static FTM: Chain = Chain {
    id: 250,
    name: "fantom",
    abi_source_url: "https://api.ftmscan.com/",
    abi_source_require_auth: true,
};

static OPTIMISM: Chain = Chain {
    id: 10,
    name: "optimism",
    abi_source_url: "https://api-optimistic.etherscan.io/",
    abi_source_require_auth: true,
};

static ARBITTUM: Chain = Chain {
    id: 42161,
    name: "arbitrum",
    abi_source_url: "https://api.arbiscan.io/",
    abi_source_require_auth: true,
};

static GNOSIS: Chain = Chain {
    id: 100,
    name: "gnosis",
    abi_source_url: "https://api.gnosisscan.io/",
    abi_source_require_auth: true,
};

static BNB_CHAIN: Chain = Chain {
    id: 56,
    name: "bsc",
    abi_source_url: "https://api.bscscan.com/",
    abi_source_require_auth: true,
};

static AVALANCHE: Chain = Chain {
    id: 43114,
    name: "avalanche",
    abi_source_url: "https://api.snowtrace.io/",
    abi_source_require_auth: true,
};

static DOGECHAIN: Chain = Chain {
    id: 2000,
    name: "dogechain",
    abi_source_url: "https://explorer.dogechain.dog/",
    abi_source_require_auth: false,
};

pub static AVAILABLE_CHAINS: [Chain; 9] = [
    ETHEREUM, POLYGON, FTM, OPTIMISM, ARBITTUM, GNOSIS, BNB_CHAIN, AVALANCHE, DOGECHAIN,
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
