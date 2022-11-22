use diesel::prelude::*;
use web3::types::{Block, Transaction, H160};

use crate::utils::{format_address, format_bytes, format_hash, format_nonce, format_number};

use super::schema::{blocks, state, txs};

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
    pub fn from_web3_block(block: &Block<Transaction>) -> Self {
        let base_fee_per_gas: String = match block.base_fee_per_gas {
            None => String::from("0"),
            Some(base_fee_per_gas) => format_number(base_fee_per_gas),
        };

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
            base_fee_per_gas,
        }
    }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = txs)]
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
}

impl DatabaseTx {
    pub fn from_web3_tx(tx: Transaction) -> Self {
        let max_fee_per_gas: String = match tx.max_fee_per_gas {
            None => String::from("0"),
            Some(max_fee_per_gas) => format_number(max_fee_per_gas),
        };

        let max_priority_fee_per_gas: String = match tx.max_priority_fee_per_gas {
            None => String::from("0"),
            Some(max_priority_fee_per_gas) => format_number(max_priority_fee_per_gas),
        };

        let to_address: String = match tx.to {
            None => format_address(H160::zero()),
            Some(to) => format_address(to),
        };

        let transaction_type: i64 = match tx.transaction_type {
            None => 0,
            Some(transaction_type) => transaction_type.as_u64() as i64,
        };

        Self {
            block_number: tx.block_number.unwrap().as_u64() as i64,
            from_address: format_address(tx.from.unwrap()),
            to_address,
            value: format_number(tx.value),
            gas_used: format_number(tx.gas),
            gas_price: format_number(tx.gas_price.unwrap()),
            hash: format_hash(tx.hash),
            transaction_index: tx.transaction_index.unwrap().as_u64() as i64,
            transaction_type,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            input: format_bytes(&tx.input),
        }
    }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = state)]
pub struct DatabaseState {
    pub id: String,
    pub last_block: i64,
}
