use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub blocks_reorg: i64,
    pub abi_source_url: &'static str,
}

impl Chain {
    pub fn new_from_borrowed(chain: &Chain) -> Self {
        Self {
            id: chain.id,
            name: chain.name,
            blocks_reorg: chain.blocks_reorg,
            abi_source_url: chain.abi_source_url,
        }
    }
}

static ETHEREUM: Chain = Chain {
    id: 1,
    name: "mainnet",
    blocks_reorg: 12,
    abi_source_url: "https://api.etherscan.io/",
};

static POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    blocks_reorg: 128,
    abi_source_url: "https://api.polygonscan.com/",
};

static FTM: Chain = Chain {
    id: 250,
    name: "fantom",
    blocks_reorg: 5,
    abi_source_url: "https://api.ftmscan.com/",
};

static OPTIMISM: Chain = Chain {
    id: 10,
    name: "optimism",
    blocks_reorg: 20,
    abi_source_url: "https://api-optimistic.etherscan.io/",
};

static ARBITTUM: Chain = Chain {
    id: 42161,
    name: "arbitrum",
    blocks_reorg: 20,
    abi_source_url: "https://api.arbiscan.io/",
};

static GNOSIS: Chain = Chain {
    id: 20,
    name: "gnosis",
    blocks_reorg: 20,
    abi_source_url: "https://api.gnosisscan.io/",
};

static BNB_CHAIN: Chain = Chain {
    id: 56,
    name: "bsc",
    blocks_reorg: 16,
    abi_source_url: "https://api.bscscan.com/",
};

static AVALANCHE: Chain = Chain {
    id: 43114,
    name: "avalanche",
    blocks_reorg: 16,
    abi_source_url: "https://api.snowtrace.io/",
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
