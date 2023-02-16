use ethers::types::{Block, Log, Transaction, TransactionReceipt, H160};
use field_count::FieldCount;

use crate::utils::{
    format_address, format_bytes, format_bytes_slice, format_hash, format_nonce, format_number,
    format_small_number,
};

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseBlock {
    pub base_fee_per_gas: String,
    pub chain: String,
    pub difficulty: String,
    pub extra_data: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub block_hash: String,
    pub logs_bloom: String,
    pub miner: String,
    pub mix_hash: String,
    pub nonce: String,
    pub number: i64,
    pub parent_hash: String,
    pub receipts_root: String,
    pub sha3_uncles: String,
    pub size: i64,
    pub state_root: String,
    pub timestamp: String,
    pub total_difficulty: String,
    pub transactions: i64,
    pub uncles: Vec<Option<String>>,
}

impl DatabaseBlock {
    pub fn from_rpc(block: &Block<Transaction>, chain: &'static str) -> Self {
        let base_fee_per_gas: String = match block.base_fee_per_gas {
            None => String::from("0"),
            Some(base_fee_per_gas) => format_number(base_fee_per_gas),
        };

        let nonce: String = match block.nonce {
            None => String::from("0"),
            Some(nonce) => format_nonce(nonce),
        };

        let uncles = block
            .uncles
            .clone()
            .into_iter()
            .map(|uncle| Some(format_hash(uncle)))
            .collect();

        let mix_hash: String = match block.mix_hash {
            None => String::from("0x"),
            Some(mix_hash) => format_hash(mix_hash),
        };

        let block_hash: String = match block.hash {
            None => String::from("0x"),
            Some(hash) => format_hash(hash),
        };

        let number: i64 = match block.number {
            None => 0,
            Some(number) => number.as_u64() as i64,
        };

        let size: i64 = match block.size {
            None => 0,
            Some(size) => size.as_u64() as i64,
        };

        let total_difficulty: String = match block.total_difficulty {
            None => String::from("0x"),
            Some(total_difficulty) => format_number(total_difficulty),
        };

        let miner: String = match block.author {
            None => format_address(H160::zero()),
            Some(author) => format_address(author),
        };

        Self {
            base_fee_per_gas,
            chain: chain.to_owned(),
            difficulty: format_number(block.difficulty),
            extra_data: format_bytes(&block.extra_data),
            gas_limit: format_number(block.gas_limit),
            gas_used: format_number(block.gas_used),
            block_hash,
            logs_bloom: format_bytes_slice(block.logs_bloom.unwrap().as_bytes()),
            miner,
            mix_hash,
            nonce,
            number,
            parent_hash: format_hash(block.parent_hash),
            receipts_root: format_hash(block.receipts_root),
            sha3_uncles: format_hash(block.uncles_hash),
            size,
            state_root: format_hash(block.state_root),
            timestamp: format_number(block.timestamp),
            total_difficulty,
            transactions: block.transactions.len() as i64,
            uncles,
        }
    }
}

pub fn byte4_from_input(input: &String) -> [u8; 4] {
    let input_sanitized = input.strip_prefix("0x").unwrap();

    if input_sanitized == "" {
        return [0x00, 0x00, 0x00, 0x00];
    }

    let input_bytes = hex::decode(input_sanitized).unwrap();

    if input_bytes.len() < 4 {
        return [0x00, 0x00, 0x00, 0x00];
    }

    let byte4: [u8; 4] = [
        input_bytes[0],
        input_bytes[1],
        input_bytes[2],
        input_bytes[3],
    ];

    return byte4;
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseTransaction {
    pub block_hash: String,
    pub block_number: i64,
    pub chain: String,
    pub from_address: String,
    pub gas: String,
    pub gas_price: String,
    pub max_priority_fee_per_gas: String,
    pub max_fee_per_gas: String,
    pub hash: String,
    pub input: String,
    pub method: String,
    pub nonce: String,
    pub timestamp: String,
    pub to_address: String,
    pub transaction_index: i64,
    pub transaction_type: i64,
    pub value: String,
}

impl DatabaseTransaction {
    pub fn from_rpc(transaction: Transaction, chain: &'static str, timestamp: String) -> Self {
        let max_fee_per_gas: String = match transaction.max_fee_per_gas {
            None => String::from("0"),
            Some(max_fee_per_gas) => format_number(max_fee_per_gas),
        };

        let max_priority_fee_per_gas: String = match transaction.max_priority_fee_per_gas {
            None => String::from("0"),
            Some(max_priority_fee_per_gas) => format_number(max_priority_fee_per_gas),
        };

        let to_address: String = match transaction.to {
            None => format_address(H160::zero()),
            Some(to) => format_address(to),
        };

        let transaction_type: i64 = match transaction.transaction_type {
            None => 0,
            Some(transaction_type) => transaction_type.as_u64() as i64,
        };

        let block_number: i64 = match transaction.block_number {
            None => 0,
            Some(block_number) => block_number.as_u64() as i64,
        };

        let block_hash: String = match transaction.block_hash {
            None => String::from("0"),
            Some(block_hash) => format_hash(block_hash),
        };

        let gas_price: String = match transaction.gas_price {
            None => String::from("0"),
            Some(gas_price) => format_number(gas_price),
        };

        let input = format_bytes(&transaction.input);

        let transaction_index: i64 = match transaction.transaction_index {
            None => 0,
            Some(transaction_index) => transaction_index.as_u64() as i64,
        };

        Self {
            block_hash,
            block_number,
            chain: chain.to_owned(),
            from_address: format_address(transaction.from),
            gas: format_number(transaction.gas),
            gas_price,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            hash: format_hash(transaction.hash),
            method: format!("0x{}", hex::encode(byte4_from_input(&input))),
            input,
            nonce: format_number(transaction.nonce),
            timestamp,
            to_address,
            transaction_index,
            transaction_type,
            value: format_number(transaction.value),
        }
    }
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseReceipt {
    pub contract_address: Option<String>,
    pub cumulative_gas_used: String,
    pub effective_gas_price: String,
    pub gas_used: String,
    pub hash: String,
    pub status: String,
}

impl DatabaseReceipt {
    pub fn from_rpc(receipt: &TransactionReceipt) -> Self {
        let contract_address: Option<String> = match receipt.contract_address {
            None => None,
            Some(contract_address) => Some(format_address(contract_address)),
        };

        let status: String = match receipt.status {
            None => String::from("-1"),
            Some(status) => format_small_number(status),
        };

        let effective_gas_price: String = match receipt.effective_gas_price {
            None => String::from("0"),
            Some(effective_gas_price) => format_number(effective_gas_price),
        };

        let gas_used: String = match receipt.gas_used {
            None => String::from("0"),
            Some(gas_used) => format_number(gas_used),
        };

        Self {
            contract_address,
            cumulative_gas_used: format_number(receipt.cumulative_gas_used),
            effective_gas_price,
            gas_used,
            hash: format_hash(receipt.transaction_hash),
            status,
        }
    }
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseLog {
    pub address: String,
    pub chain: String,
    pub data: String,
    pub erc20_transfers_parsed: bool,
    pub hash: String,
    pub log_index: i64,
    pub removed: bool,
    pub topics: Vec<Option<String>>,
}

impl DatabaseLog {
    pub fn from_rpc(log: Log, chain: String) -> Self {
        let hash: String = match log.transaction_hash {
            None => String::from("0"),
            Some(hash) => format_hash(hash),
        };

        let log_index: i64 = match log.log_index {
            None => 0,
            Some(log_index) => log_index.as_u64() as i64,
        };

        let removed: bool = match log.removed {
            None => false,
            Some(removed) => removed,
        };

        Self {
            address: format_address(log.address),
            chain,
            topics: log
                .topics
                .clone()
                .into_iter()
                .map(|topic| Some(format_hash(topic)))
                .collect(),
            data: format_bytes(&log.data),
            hash,
            log_index,
            removed,
            erc20_transfers_parsed: false,
        }
    }
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseMethod {
    pub method: String,
    pub name: String,
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseContractInformation {
    pub chain: String,
    pub contract: String,
    pub abi: Option<String>,
    pub name: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseContract {
    pub block: i64,
    pub chain: String,
    pub contract: String,
    pub creator: String,
    pub hash: String,
    pub parsed: bool,
    pub verified: bool,
}

impl DatabaseContract {
    pub fn from_rpc(receipt: TransactionReceipt, chain: &'static str) -> Self {
        let block_number: i64 = match receipt.block_number {
            None => 0,
            Some(block_number) => block_number.as_u64() as i64,
        };

        let contract_address: String = match receipt.contract_address {
            None => format_address(H160::zero()),
            Some(contract_address) => format_address(contract_address),
        };

        Self {
            block: block_number,
            chain: chain.to_owned(),
            contract: contract_address,
            creator: format_address(receipt.from),
            hash: format_hash(receipt.transaction_hash),
            parsed: false,
            verified: false,
        }
    }
}

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseChainIndexedState {
    pub chain: String,
    pub indexed_blocks_amount: i64,
}
