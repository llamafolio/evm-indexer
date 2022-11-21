// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (number) {
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
        transaction_index -> Varchar,
        transaction_type -> Nullable<Varchar>,
        max_fee_per_gas -> Nullable<Varchar>,
        max_priority_fee_per_gas -> Nullable<Varchar>,
        input -> Varchar,
        timestamp -> Varchar,
        success -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    txs,
);
