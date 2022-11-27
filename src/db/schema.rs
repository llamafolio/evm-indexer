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
    logs (hash) {
        hash -> Varchar,
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
    txs (hash) {
        hash -> Varchar,
        block_number -> Int8,
        from_address -> Varchar,
        to_address -> Varchar,
        value -> Varchar,
        gas_used -> Varchar,
        gas_price -> Varchar,
        transaction_index -> Int8,
        transaction_type -> Nullable<Int8>,
        max_fee_per_gas -> Nullable<Varchar>,
        max_priority_fee_per_gas -> Nullable<Varchar>,
        input -> Varchar,
        chain -> Varchar,
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
    contract_creations,
    contract_interactions,
    logs,
    token_transfers,
    txs,
    txs_receipts,
);
