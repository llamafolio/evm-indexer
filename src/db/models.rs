use anyhow::bail;
use diesel::prelude::*;
use ethabi::ParamType;
use field_count::FieldCount;
use reth_primitives::{
    rpc::{Block, Log, Transaction, TransactionReceipt},
    H160,
};

use crate::utils::{
    format_address, format_bool, format_bytes, format_hash, format_nonce, format_number,
};

use super::schema::{
    blocks, contract_abis, contract_creations, contract_interactions, contracts_adapters,
    excluded_tokens, logs, method_ids, state, token_transfers, tokens, txs, txs_no_receipt,
    txs_receipts,
};

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
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
    pub chain: String,
}

impl DatabaseBlock {
    pub fn from_web3(block: &Block<Transaction>, chain: String) -> Self {
        let base_fee_per_gas: String = match block.base_fee_per_gas {
            None => String::from("0"),
            Some(base_fee_per_gas) => format_number(base_fee_per_gas),
        };

        let nonce: String = match block.nonce {
            None => String::from("0"),
            Some(nonce) => format_nonce(nonce),
        };

        Self {
            number: block.number.unwrap().as_u64() as i64,
            hash: format_hash(block.hash.unwrap()),
            difficulty: format_number(block.difficulty),
            total_difficulty: format_number(block.total_difficulty.unwrap()),
            miner: format_address(block.author.unwrap()),
            gas_limit: format_number(block.gas_limit),
            gas_used: format_number(block.gas_used),
            txs: block.transactions.len() as i64,
            timestamp: format_number(block.timestamp),
            size: format_number(block.size.unwrap()),
            nonce,
            base_fee_per_gas,
            chain,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = txs)]
pub struct DatabaseTx {
    pub block_number: i64,
    pub timestamp: String,
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
    pub chain: String,
    pub method_id: String,
}

impl DatabaseTx {
    pub fn from_web3(tx: &Transaction, chain: String, timestamp: Option<&String>) -> Self {
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

        let empty_timestamp = String::from("");

        let parsed_timestamp: &String = match timestamp {
            None => &empty_timestamp,
            Some(timestamp) => timestamp,
        };

        let input = format_bytes(&tx.input);

        Self {
            block_number: tx.block_number.unwrap().as_u64() as i64,
            from_address: format_address(tx.from),
            to_address,
            value: format_number(tx.value),
            gas_used: format_number(tx.gas),
            gas_price: format_number(tx.gas_price.unwrap()),
            hash: format_hash(tx.hash),
            transaction_index: tx.transaction_index.unwrap().as_u64() as i64,
            transaction_type,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            input: input.clone(),
            chain,
            timestamp: parsed_timestamp.to_string(),
            method_id: format!("0x{}", hex::encode(byte4_from_input(&input))),
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = txs_receipts)]
pub struct DatabaseTxReceipt {
    pub hash: String,
    pub success: bool,
    pub chain: String,
}

impl DatabaseTxReceipt {
    pub fn from_web3(receipt: TransactionReceipt, chain: String) -> Self {
        let success: bool = match receipt.status {
            None => true,
            Some(success) => format_bool(success),
        };

        Self {
            hash: format_hash(receipt.transaction_hash),
            success,
            chain,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = logs)]
pub struct DatabaseTxLogs {
    pub hash_with_index: String,
    pub hash: String,
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub log_index: i64,
    pub transaction_log_index: i64,
    pub log_type: String,
    pub chain: String,
}

impl DatabaseTxLogs {}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = contract_interactions)]
pub struct DatabaseContractInteraction {
    pub hash: String,
    pub block: i64,
    pub address: String,
    pub contract: String,
    pub chain: String,
}

impl DatabaseContractInteraction {
    pub fn from_transaction(tx: &DatabaseTx, chain: String) -> Self {
        Self {
            hash: tx.hash.clone(),
            block: tx.block_number,
            address: tx.from_address.clone(),
            contract: tx.to_address.clone(),
            chain,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = contract_creations)]
pub struct DatabaseContractCreation {
    pub hash: String,
    pub block: i64,
    pub contract: String,
    pub chain: String,
}

impl DatabaseContractCreation {
    pub fn from_receipt(receipt: &TransactionReceipt, chain: String, contract: String) -> Self {
        Self {
            hash: format_hash(receipt.transaction_hash),
            block: receipt.block_number.unwrap().as_u64() as i64,
            contract,
            chain,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = token_transfers)]
pub struct DatabaseTokenTransfers {
    pub hash_with_index: String,
    pub hash: String,
    pub block: i64,
    pub token: String,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub log_index: i64,
    pub chain: String,
}

pub fn token_transfers_from_logs(
    log: &Log,
    receipt: &TransactionReceipt,
    chain: String,
) -> anyhow::Result<DatabaseTokenTransfers> {
    if log.topics.len() != 3 {
        bail!("No topics for log");
    }

    let event = ethabi::Event {
        name: "Transfer".to_owned(),
        inputs: vec![
            ethabi::EventParam {
                name: "from".to_owned(),
                kind: ParamType::Address,
                indexed: false,
            },
            ethabi::EventParam {
                name: "to".to_owned(),
                kind: ParamType::Address,
                indexed: false,
            },
            ethabi::EventParam {
                name: "amount".to_owned(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
        ],
        anonymous: false,
    };

    // Check the first topic against keccak256(Transfer(address,address,uint256))
    if format_hash(log.topics[0]) != format!("{:?}", event.signature()) {
        bail!("Topic doesn't match the Transfer event");
    }

    let from_address: String = match ethabi::decode(&[ParamType::Address], log.topics[1].as_bytes())
    {
        Ok(address) => {
            if address.len() == 0 {
                bail!("From address not found");
            } else {
                format!("{:?}", address[0].clone().into_address().unwrap())
            }
        }
        Err(_) => bail!("Topic doesn't include the from_address"),
    };

    let to_address = match ethabi::decode(&[ParamType::Address], log.topics[2].as_bytes()) {
        Ok(address) => {
            if address.len() == 0 {
                bail!("To address not found");
            } else {
                format!("{:?}", address[0].clone().into_address().unwrap())
            }
        }
        Err(_) => bail!("Topic doesn't include the to_address"),
    };

    let value = match ethabi::decode(&[ParamType::Uint(256)], &log.data.0[..]) {
        Ok(value) => {
            if value.len() == 0 {
                bail!("Value not found");
            } else {
                format!("{:?}", value[0].clone().into_uint().unwrap())
            }
        }
        Err(_) => bail!("Unable to decode value"),
    };

    return Ok(DatabaseTokenTransfers {
        hash_with_index: format!(
            "{}-{}",
            format_hash(receipt.transaction_hash),
            log.log_index.unwrap().as_u64()
        ),
        hash: format_hash(receipt.transaction_hash),
        block: receipt.block_number.unwrap().as_u64() as i64,
        token: format_address(log.address),
        from_address,
        to_address,
        value,
        log_index: log.log_index.unwrap().as_u64() as i64,
        chain,
    });
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = state)]
pub struct DatabaseState {
    pub chain: String,
    pub blocks: i64,
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = tokens)]
pub struct DatabaseToken {
    pub address_with_chain: String,
    pub address: String,
    pub chain: String,
    pub name: String,
    pub decimals: i64,
    pub symbol: String,
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = excluded_tokens)]
pub struct DatabaseExcludedToken {
    pub address_with_chain: String,
    pub address: String,
    pub chain: String,
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = txs_no_receipt)]
pub struct DatabaseTxNoReceipt {
    pub hash: String,
    pub chain: String,
    pub block_number: i64,
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = contract_abis)]
pub struct DatabaseContractABI {
    pub address_with_chain: String,
    pub chain: String,
    pub address: String,
    pub abi: Option<String>,
    pub verified: bool,
}

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = method_ids)]
pub struct DatabaseMethodID {
    pub method_id: String,
    pub name: String,
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

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = contracts_adapters)]
pub struct DatabaseContractAdapter {
    pub address_with_chain: String,
    pub adapter_id: String,
    pub chain: String,
    pub address: String,
}
