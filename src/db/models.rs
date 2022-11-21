use diesel::prelude::*;

#[derive(Queryable)]
pub struct DatabaseBlock {
    pub number: i64,
    pub hash: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub miner: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub txs: i64,
    pub timestamp: i64,
    pub size: i64,
    pub nonce: String,
    pub base_fee_per_gas: String,
}

#[derive(Queryable)]
pub struct DatabaseTx {
    pub block_number: i64,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub gas_used: String,
    pub gas_price: String,
    pub hash: String,
    pub transaction_index: i64,
    pub transaction_type: i64,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub input: String,
    pub timestamp: i64,
    pub success: bool,
}
