use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub block_explorer: &'static str,
    pub abi_source_api: &'static str,
    pub abi_source_require_auth: bool,
    pub supports_blocks_receipts: bool,
}

impl Chain {
    pub fn new_from_borrowed(chain: &Chain) -> Self {
        Self {
            id: chain.id,
            name: chain.name,
            block_explorer: chain.block_explorer,
            abi_source_api: chain.abi_source_api,
            abi_source_require_auth: chain.abi_source_require_auth,
            supports_blocks_receipts: chain.supports_blocks_receipts,
        }
    }
}

pub const ETHEREUM: Chain = Chain {
    id: 1,
    name: "ethereum",
    block_explorer: "https://etherscan.io/",
    abi_source_api: "https://api.etherscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: true,
};

pub const POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    block_explorer: "https://polygonscan.com/",
    abi_source_api: "https://api.polygonscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: true,
};

pub const FANTOM: Chain = Chain {
    id: 250,
    name: "fantom",
    block_explorer: "https://ftmscan.com/",
    abi_source_api: "https://api.ftmscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const BSC: Chain = Chain {
    id: 56,
    name: "bsc",
    block_explorer: "https://bscscan.com/",
    abi_source_api: "https://api.bscscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: true,
};

pub const GNOSIS: Chain = Chain {
    id: 100,
    name: "gnosis",
    block_explorer: "https://gnosisscan.io/",
    abi_source_api: "https://api.gnosisscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const OPTIMISM: Chain = Chain {
    id: 10,
    name: "optimism",
    block_explorer: "https://optimistic.etherscan.io/",
    abi_source_api: "https://api-optimistic.etherscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const ARBITRUM_ONE: Chain = Chain {
    id: 42161,
    name: "arbitrum",
    block_explorer: "https://arbiscan.io/",
    abi_source_api: "https://api.arbiscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const ARBITRUM_NOVA: Chain = Chain {
    id: 42170,
    name: "arbitrum-nova",
    block_explorer: "https://nova.arbiscan.io/",
    abi_source_api: "https://nova-api.arbiscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const MOONBEAM: Chain = Chain {
    id: 1284,
    name: "moonbeam",
    block_explorer: "https://moonscan.io/",
    abi_source_api: "https://api.moonscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const AVALANCHE: Chain = Chain {
    id: 43114,
    name: "avalanche",
    block_explorer: "https://snowtrace.io/",
    abi_source_api: "https://api.snowtrace.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const BITTORRENT: Chain = Chain {
    id: 199,
    name: "bittorrent",
    block_explorer: "https://bttcscan.com/",
    abi_source_api: "https://api.bttcscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub const CELO: Chain = Chain {
    id: 42220,
    name: "celo",
    block_explorer: "https://celoscan.io/",
    abi_source_api: "https://api.celoscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
};

pub static CHAINS: [Chain; 12] = [
    ETHEREUM,
    POLYGON,
    FANTOM,
    BSC,
    GNOSIS,
    OPTIMISM,
    ARBITRUM_ONE,
    ARBITRUM_NOVA,
    MOONBEAM,
    AVALANCHE,
    BITTORRENT,
    CELO,
];

pub fn get_chains() -> HashMap<String, Chain> {
    let mut chains: HashMap<String, Chain> = HashMap::new();

    for chain in CHAINS.into_iter() {
        chains.insert(String::from(chain.name), chain);
    }

    return chains;
}

pub fn get_chain(chain: String) -> Chain {
    let chains = get_chains();

    let selected_chain = chains.get(&chain).expect("Chain not found");

    return Chain::new_from_borrowed(selected_chain);
}
