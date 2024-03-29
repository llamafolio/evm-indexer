use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
    pub block_explorer: &'static str,
    pub abi_source_api: &'static str,
    pub abi_source_require_auth: bool,
    pub supports_blocks_receipts: bool,
    pub public_rpc: &'static str,
    pub tokens_lists: &'static [&'static str],
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
            public_rpc: chain.public_rpc,
            tokens_lists: chain.tokens_lists,
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
    public_rpc: "https://eth.llamarpc.com",
    tokens_lists: &["https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/ethereum/tokenlist.json", "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/ethereum.json"],
};

pub const POLYGON: Chain = Chain {
    id: 137,
    name: "polygon",
    block_explorer: "https://polygonscan.com/",
    abi_source_api: "https://api.polygonscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: true,
    public_rpc: "https://polygon.llamarpc.com",
    tokens_lists: &["https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/polygon/tokenlist.json", "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/polygon.json"],
};

pub const FANTOM: Chain = Chain {
    id: 250,
    name: "fantom",
    block_explorer: "https://ftmscan.com/",
    abi_source_api: "https://api.ftmscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ftm.tools",
    tokens_lists: &["https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/fantom/tokenlist.json", "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/ftm.json"],
};

pub const BSC: Chain = Chain {
    id: 56,
    name: "bsc",
    block_explorer: "https://bscscan.com/",
    abi_source_api: "https://api.bscscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: true,
    public_rpc: "https://bscrpc.com",
    tokens_lists: &[
        "https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/bsc/tokenlist.json",
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/bsc.json",
    ],
};

pub const GNOSIS: Chain = Chain {
    id: 100,
    name: "gnosis",
    block_explorer: "https://gnosisscan.io/",
    abi_source_api: "https://api.gnosisscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ankr.com/gnosis",
    tokens_lists: &[
        "https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/xdai/tokenlist.json",
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/gnosis.json",
    ],
};

pub const OPTIMISM: Chain = Chain {
    id: 10,
    name: "optimism",
    block_explorer: "https://optimistic.etherscan.io/",
    abi_source_api: "https://api-optimistic.etherscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ankr.com/optimism",
    tokens_lists: &[
        "https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/optimism/tokenlist.json",
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/optimism.json",
    ],
};

pub const ARBITRUM_ONE: Chain = Chain {
    id: 42161,
    name: "arbitrum",
    block_explorer: "https://arbiscan.io/",
    abi_source_api: "https://api.arbiscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ankr.com/arbitrum",
    tokens_lists: &[
        "https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/arbitrum/tokenlist.json",
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/arbitrum.json",
    ],
};

pub const ARBITRUM_NOVA: Chain = Chain {
    id: 42170,
    name: "arbitrum-nova",
    block_explorer: "https://nova.arbiscan.io/",
    abi_source_api: "https://api-nova.arbiscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://nova.arbitrum.io/rpc",
    tokens_lists: &[],
};

pub const MOONBEAM: Chain = Chain {
    id: 1284,
    name: "moonbeam",
    block_explorer: "https://moonscan.io/",
    abi_source_api: "https://api.moonscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ankr.com/moonbeam",
    tokens_lists: &[
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/moonbeam.json",
    ],
};

pub const AVALANCHE: Chain = Chain {
    id: 43114,
    name: "avalanche",
    block_explorer: "https://snowtrace.io/",
    abi_source_api: "https://api.snowtrace.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ankr.com/avalanche",
    tokens_lists: &[
        "https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/avax/tokenlist.json",
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/avax.json",
    ],
};

pub const BITTORRENT: Chain = Chain {
    id: 199,
    name: "bittorrent",
    block_explorer: "https://bttcscan.com/",
    abi_source_api: "https://api.bttcscan.com/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.bittorrentchain.io",
    tokens_lists: &[],
};

pub const CELO: Chain = Chain {
    id: 42220,
    name: "celo",
    block_explorer: "https://celoscan.io/",
    abi_source_api: "https://api.celoscan.io/",
    abi_source_require_auth: true,
    supports_blocks_receipts: false,
    public_rpc: "https://rpc.ankr.com/celo",
    tokens_lists: &[
        "https://raw.githubusercontent.com/llamafolio/llamafolio-tokens/master/celo/tokenlist.json",
        "https://raw.githubusercontent.com/viaprotocol/tokenlists/main/tokenlists/celo.json",
    ],
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
