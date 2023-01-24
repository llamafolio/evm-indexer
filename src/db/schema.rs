// @generated automatically by Diesel CLI.

diesel::table! {
    abis (contract, chain) {
        chain -> Text,
        contract -> Text,
        abi -> Nullable<Text>,
        verified -> Bool,
    }
}

diesel::table! {
    blocks (block_hash) {
        base_fee_per_gas -> Text,
        chain -> Text,
        difficulty -> Text,
        extra_data -> Text,
        gas_limit -> Text,
        gas_used -> Text,
        block_hash -> Text,
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
    contracts (hash) {
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
        address -> Varchar,
        chain -> Varchar,
        adapter_id -> Varchar,
    }
}

diesel::table! {
    contracts_interactions (hash) {
        hash -> Text,
        block -> Int8,
        address -> Text,
        contract -> Text,
        chain -> Text,
    }
}

diesel::table! {
    erc20_balances (address, token, chain) {
        address -> Text,
        chain -> Text,
        token -> Text,
        balance -> Text,
    }
}

diesel::table! {
    erc20_tokens (address, chain) {
        address -> Text,
        chain -> Text,
        name -> Nullable<Text>,
        decimals -> Nullable<Int8>,
        symbol -> Nullable<Text>,
    }
}

diesel::table! {
    erc20_transfers (hash, log_index) {
        chain -> Text,
        hash -> Text,
        log_index -> Int8,
        token -> Text,
        from_address -> Text,
        to_address -> Text,
        value -> Text,
        erc20_tokens_parsed -> Bool,
        erc20_balances_parsed -> Bool,
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
    }
}

diesel::table! {
    methods (method) {
        method -> Text,
        name -> Text,
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
        max_priority_fee_per_gas -> Nullable<Text>,
        max_fee_per_gas -> Nullable<Text>,
        hash -> Text,
        input -> Text,
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
    contracts_interactions,
    erc20_balances,
    erc20_tokens,
    erc20_transfers,
    logs,
    methods,
    receipts,
    transactions,
);
