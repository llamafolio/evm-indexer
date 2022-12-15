// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (hash) {
        number -> Int8,
        hash -> Varchar,
        difficulty -> Varchar,
        total_difficulty -> Varchar,
        miner -> Varchar,
        gas_limit -> Varchar,
        gas_used -> Varchar,
        txs -> Int8,
        timestamp -> Varchar,
        size -> Varchar,
        nonce -> Varchar,
        base_fee_per_gas -> Varchar,
        chain -> Varchar,
    }
}

diesel::table! {
    contract_abis (address_with_chain) {
        address_with_chain -> Varchar,
        chain -> Varchar,
        address -> Varchar,
        abi -> Nullable<Varchar>,
        verified -> Nullable<Bool>,
    }
}

diesel::table! {
    contract_creations (hash) {
        hash -> Varchar,
        block -> Int8,
        contract -> Varchar,
        chain -> Varchar,
    }
}

diesel::table! {
    contract_interactions (hash) {
        hash -> Varchar,
        block -> Int8,
        address -> Varchar,
        contract -> Varchar,
        chain -> Varchar,
    }
}

diesel::table! {
    contracts_adapters (address_with_chain) {
        address_with_chain -> Varchar,
        address -> Varchar,
        chain -> Varchar,
        adapter_id -> Varchar,
    }
}

diesel::table! {
    excluded_tokens (address_with_chain) {
        address_with_chain -> Varchar,
        address -> Varchar,
        chain -> Varchar,
    }
}

diesel::table! {
    logs (hash_with_index) {
        hash_with_index -> Varchar,
        hash -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        topics -> Nullable<Array<Nullable<Text>>>,
        data -> Nullable<Varchar>,
        log_index -> Nullable<Int8>,
        transaction_log_index -> Nullable<Int8>,
        log_type -> Nullable<Varchar>,
        chain -> Varchar,
    }
}

diesel::table! {
    method_ids (method_id) {
        method_id -> Varchar,
        name -> Varchar,
    }
}

diesel::table! {
    state (chain) {
        chain -> Varchar,
        blocks -> Int8,
    }
}

diesel::table! {
    token_transfers (hash_with_index) {
        hash_with_index -> Varchar,
        hash -> Varchar,
        log_index -> Int8,
        block -> Int8,
        token -> Varchar,
        from_address -> Varchar,
        to_address -> Varchar,
        value -> Varchar,
        chain -> Varchar,
    }
}

diesel::table! {
    tokens (address_with_chain) {
        address_with_chain -> Varchar,
        address -> Varchar,
        chain -> Varchar,
        name -> Varchar,
        decimals -> Int8,
        symbol -> Varchar,
    }
}

diesel::table! {
    txs (hash) {
        hash -> Varchar,
        block_number -> Int8,
        from_address -> Varchar,
        to_address -> Varchar,
        value -> Varchar,
        gas_used -> Varchar,
        gas_price -> Varchar,
        timestamp -> Varchar,
        transaction_index -> Int8,
        transaction_type -> Nullable<Int8>,
        max_fee_per_gas -> Nullable<Varchar>,
        max_priority_fee_per_gas -> Nullable<Varchar>,
        input -> Varchar,
        method_id -> Varchar,
        chain -> Varchar,
    }
}

diesel::table! {
    txs_no_receipt (hash) {
        hash -> Varchar,
        chain -> Varchar,
        block_number -> Int8,
    }
}

diesel::table! {
    txs_receipts (hash) {
        hash -> Varchar,
        success -> Nullable<Bool>,
        chain -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    contract_abis,
    contract_creations,
    contract_interactions,
    contracts_adapters,
    excluded_tokens,
    logs,
    method_ids,
    state,
    token_transfers,
    tokens,
    txs,
    txs_no_receipt,
    txs_receipts,
);
