// @generated automatically by Diesel CLI.

diesel::table! {
    abis (contract, chain) {
        abi -> Nullable<Text>,
        chain -> Text,
        contract -> Text,
        verified -> Bool,
    }
}

diesel::table! {
    blocks (block_hash) {
        base_fee_per_gas -> Text,
        block_hash -> Text,
        chain -> Text,
        difficulty -> Text,
        extra_data -> Text,
        gas_limit -> Text,
        gas_used -> Text,
        logs_bloom -> Text,
        miner -> Text,
        mix_hash -> Text,
        nonce -> Text,
        number -> Int8,
        parent_hash -> Text,
        receipts_root -> Text,
        sha3_uncles -> Text,
        size -> Int8,
        state_root -> Text,
        timestamp -> Text,
        total_difficulty -> Text,
        transactions -> Int8,
        uncles -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    chains_indexed_state (chain) {
        chain -> Text,
        indexed_blocks_amount -> Int8,
    }
}

diesel::table! {
    contracts (contract, chain) {
        block -> Int8,
        chain -> Text,
        contract -> Text,
        creator -> Text,
        hash -> Text,
        parsed -> Bool,
        verified -> Bool,
    }
}

diesel::table! {
    contracts_adapters (address, chain) {
        adapter_id -> Text,
        address -> Text,
        chain -> Text,
    }
}

diesel::table! {
    erc20_balances (address, token, chain) {
        address -> Text,
        balance -> Float8,
        chain -> Text,
        token -> Text,
    }
}

diesel::table! {
    erc20_tokens (address, chain) {
        address -> Text,
        chain -> Text,
        decimals -> Nullable<Int8>,
        name -> Nullable<Text>,
        symbol -> Nullable<Text>,
    }
}

diesel::table! {
    erc20_transfers (hash, log_index) {
        chain -> Text,
        erc20_balances_parsed -> Bool,
        erc20_tokens_parsed -> Bool,
        from_address -> Text,
        hash -> Text,
        log_index -> Int8,
        to_address -> Text,
        token -> Text,
        value -> Text,
    }
}

diesel::table! {
    logs (hash, log_index) {
        address -> Text,
        chain -> Text,
        data -> Text,
        erc20_transfers_parsed -> Bool,
        hash -> Text,
        log_index -> Int8,
        removed -> Bool,
        topics -> Array<Nullable<Text>>,
        nft_transfers_parsed -> Bool,
    }
}

diesel::table! {
    methods (method) {
        method -> Text,
        name -> Text,
    }
}

diesel::table! {
    nft_balances (address, token, chain) {
        address -> Text,
        chain -> Text,
        token -> Text,
        token_id -> Numeric,
        balance -> Numeric,
    }
}

diesel::table! {
    nft_token_uris (token, token_id, chain) {
        token -> Text,
        token_id -> Numeric,
        chain -> Text,
        token_uri -> Nullable<Text>,
        is_parsed -> Bool,
    }
}

diesel::table! {
    nft_tokens (address, chain) {
        address -> Text,
        chain -> Text,
        nft_type -> Text,
        name -> Nullable<Text>,
        symbol -> Nullable<Text>,
        contract_uri -> Nullable<Text>,
    }
}

diesel::table! {
    nft_transfers (hash, log_index, transfer_index) {
        chain -> Text,
        nft_balances_parsed -> Bool,
        nft_tokens_parsed -> Bool,
        from_address -> Text,
        to_address -> Text,
        token -> Text,
        token_id -> Numeric,
        value -> Numeric,
        hash -> Text,
        log_index -> Int8,
        transfer_index -> Int8,
        transfer_type -> Text,
    }
}

diesel::table! {
    receipts (hash) {
        contract_address -> Nullable<Text>,
        cumulative_gas_used -> Text,
        effective_gas_price -> Text,
        gas_used -> Text,
        hash -> Text,
        status -> Text,
    }
}

diesel::table! {
    transactions (hash) {
        block_hash -> Text,
        block_number -> Int8,
        chain -> Text,
        from_address -> Text,
        gas -> Text,
        gas_price -> Text,
        hash -> Text,
        input -> Text,
        max_fee_per_gas -> Nullable<Text>,
        max_priority_fee_per_gas -> Nullable<Text>,
        method -> Text,
        nonce -> Text,
        timestamp -> Text,
        to_address -> Text,
        transaction_index -> Int8,
        transaction_type -> Nullable<Int8>,
        value -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    abis,
    blocks,
    chains_indexed_state,
    contracts,
    contracts_adapters,
    erc20_balances,
    erc20_tokens,
    erc20_transfers,
    logs,
    methods,
    nft_balances,
    nft_token_uris,
    nft_tokens,
    nft_transfers,
    receipts,
    transactions,
);