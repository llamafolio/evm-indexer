// @generated automatically by Diesel CLI.

diesel::table! {
    chains_indexed_state (chain) {
        chain -> Text,
        indexed_blocks_amount -> Int8,
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
    evm_abis (contract, chain) {
        chain -> Text,
        contract -> Text,
        abi -> Nullable<Text>,
        verified -> Bool,
    }
}

diesel::table! {
    evm_blocks (block_hash) {
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
    evm_contracts (hash) {
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
    evm_contracts_interactions (hash) {
        hash -> Text,
        block -> Int8,
        address -> Text,
        contract -> Text,
        chain -> Text,
    }
}

diesel::table! {
    evm_erc20_balances (address, token, chain) {
        address -> Text,
        chain -> Text,
        token -> Text,
        sent -> Text,
        received -> Text,
    }
}

diesel::table! {
    evm_erc20_tokens (address, chain) {
        address -> Text,
        chain -> Text,
        name -> Nullable<Text>,
        decimals -> Nullable<Int8>,
        symbol -> Nullable<Text>,
    }
}

diesel::table! {
    evm_erc20_transfers (hash, log_index) {
        hash -> Text,
        log_index -> Int8,
        token -> Text,
        from_address -> Text,
        to_address -> Text,
        value -> Text,
        erc20_tokens_parsed -> Nullable<Bool>,
        erc20_balances_parsed -> Nullable<Bool>,
    }
}

diesel::table! {
    evm_methods (method) {
        method -> Text,
        name -> Text,
    }
}

diesel::table! {
    evm_transactions (hash) {
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

diesel::table! {
    evm_transactions_logs (hash, log_index) {
        address -> Text,
        topics -> Array<Nullable<Text>>,
        data -> Text,
        hash -> Text,
        log_index -> Int8,
        removed -> Bool,
        erc20_transfers_parsed -> Nullable<Bool>,
    }
}

diesel::table! {
    evm_transactions_receipts (hash) {
        contract_address -> Nullable<Text>,
        cumulative_gas_used -> Text,
        effective_gas_price -> Text,
        gas_used -> Text,
        hash -> Text,
        status -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    chains_indexed_state,
    contracts_adapters,
    evm_abis,
    evm_blocks,
    evm_contracts,
    evm_contracts_interactions,
    evm_erc20_balances,
    evm_erc20_tokens,
    evm_erc20_transfers,
    evm_methods,
    evm_transactions,
    evm_transactions_logs,
    evm_transactions_receipts,
);
