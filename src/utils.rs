use serde::Deserialize;
use web3::types::{Block, Bytes, Transaction, TransactionReceipt, H160, H256, H64, U256, U64};

pub fn format_nonce(h: H64) -> String {
    return format!("{:?}", h);
}

pub fn format_bool(h: U64) -> bool {
    let data = format!("{:?}", h);
    return data == "1";
}

pub fn format_hash(h: H256) -> String {
    return format!("{:?}", h);
}

pub fn format_address(h: H160) -> String {
    return format!("{:?}", h);
}

pub fn format_bytes(b: &Bytes) -> String {
    return format!("{}", serde_json::to_string(b).unwrap().replace("\"", ""));
}

pub fn format_number(n: U256) -> String {
    return format!("{}", n.to_string());
}

pub fn format_block(b: serde_json::Value) -> Result<Block<Transaction>, serde_json::Error> {
    return serde_json::from_value(b);
}

pub fn format_receipt(b: serde_json::Value) -> Result<TransactionReceipt, serde_json::Error> {
    return serde_json::from_value(b);
}

#[derive(Deserialize, Debug)]
struct ReceiptsMap {
    receipts: Vec<TransactionReceipt>,
}

pub fn format_receipts(b: serde_json::Value) -> Vec<TransactionReceipt> {
    let receipts_res: Result<Vec<TransactionReceipt>, serde_json::Error> =
        serde_json::from_value(b.clone());

    match receipts_res {
        Ok(receipts) => return receipts,
        Err(_) => {
            let object: Result<ReceiptsMap, serde_json::Error> = serde_json::from_value(b.clone());
            match object {
                Ok(receipts) => return receipts.receipts,
                Err(_) => {
                    return Vec::new();
                }
            }
        }
    }
}

pub static ERC20_ABI: &[u8] = include_bytes!("res/abi/erc20.json").as_slice();
