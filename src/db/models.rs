use diesel::prelude::*;
use web3::types::{Block, Transaction};

use crate::utils::{format_address, format_hash, format_nonce, format_number};

use super::schema::blocks;

#[derive(Queryable, Insertable)]
#[diesel(table_name = blocks)]
pub struct DatabaseBlock {
    pub number: i64,
    pub hash: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub miner: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub txs: i64,
    pub timestamp: String,
    pub size: String,
    pub nonce: String,
    pub base_fee_per_gas: String,
}

impl DatabaseBlock {
    pub fn from_web3_block(block: Block<Transaction>) -> Self {
        Self {
            number: block.number.unwrap().as_u64() as i64,
            hash: format_hash(block.hash.unwrap()),
            difficulty: format_number(block.difficulty),
            total_difficulty: format_number(block.total_difficulty.unwrap()),
            miner: format_address(block.author),
            gas_limit: format_number(block.gas_limit),
            gas_used: format_number(block.gas_used),
            txs: block.transactions.len() as i64,
            timestamp: format_number(block.timestamp),
            size: format_number(block.size.unwrap()),
            nonce: format_nonce(block.nonce.unwrap()),
            base_fee_per_gas: format_number(block.base_fee_per_gas.unwrap()),
        }
    }
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
