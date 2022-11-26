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

pub fn format_block(b: serde_json::Value) -> Block<Transaction> {
    return serde_json::from_value(b).unwrap();
}

pub fn format_receipt(b: serde_json::Value) -> TransactionReceipt {
    return serde_json::from_value(b).unwrap();
}

pub fn format_receipts(b: serde_json::Value) -> Vec<TransactionReceipt> {
    return serde_json::from_value(b).unwrap();
}
