use std::str::FromStr;

use crate::db::{
    db::{get_chunks, Database},
    models::models::DatabaseLog,
    schema::{nft_transfers, logs},
};
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::{prelude::*, result::Error};
use ethabi::{ethereum_types::H256, ParamType};
use ethers::types::Bytes;
use field_count::FieldCount;
use log::info;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = nft_transfers)]
pub struct DatabaseNftTransfer {
    pub chain: String,
    pub nft_balances_parsed: bool,
    pub nft_tokens_parsed: bool,
    pub from_address: String,
    pub to_address: String,
    pub token: String,
    pub token_id: BigDecimal,
    pub value: BigDecimal,
    pub hash: String,
    pub log_index: i64,
    pub transfer_index: i64,
    pub transfer_type: String,
}

pub struct NftTransfers {}

impl NftTransfers {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseLog>> {
        let mut connection = db.establish_connection();

        let logs: Result<Vec<DatabaseLog>, Error> = logs::table
            .select(logs::all_columns)
            .filter(
                logs::nft_transfers_parsed
                    .is_null()
                    .or(logs::nft_transfers_parsed.eq(false)),
            )
            .limit(50000)
            .load::<DatabaseLog>(&mut connection);

        match logs {
            Ok(logs) => Ok(logs),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, logs: &Vec<DatabaseLog>) -> Result<()> {
        let mut db_nft_transfers = Vec::new();

        let mut db_parsed_logs = Vec::new();

        for log in logs {
            let mut parsed_log = log.to_owned();

            parsed_log.nft_transfers_parsed = true;

            db_parsed_logs.push(parsed_log);

            let event721 = ethabi::Event {
                name: "Transfer".to_owned(),
                inputs: vec![
                    ethabi::EventParam {
                        name: "from".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "tokenId".to_owned(),
                        kind: ParamType::Uint(256),
                        indexed: true,
                    },
                ],
                anonymous: false,
            };
        
            let event1155single = ethabi::Event {
                name: "TransferSingle".to_owned(),
                inputs: vec![
                    ethabi::EventParam {
                        name: "operator".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "from".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "id".to_owned(),
                        kind: ParamType::Uint(256),
                        indexed: false,
                    },
                    ethabi::EventParam {
                        name: "value".to_owned(),
                        kind: ParamType::Uint(256),
                        indexed: false,
                    },
                ],
                anonymous: false,
            };
        
            let event1155batch = ethabi::Event {
                name: "TransferBatch".to_owned(),
                inputs: vec![
                    ethabi::EventParam {
                        name: "operator".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "from".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "ids".to_owned(),
                        kind: ParamType::Array(Box::new(ParamType::Uint(256))),
                        indexed: false,
                    },
                    ethabi::EventParam {
                        name: "values".to_owned(),
                        kind: ParamType::Array(Box::new(ParamType::Uint(256))),
                        indexed: false,
                    },
                ],
                anonymous: false,
            };

            let topic_1 = log.topics[0].clone().unwrap();

            // Check the first topic against
            if topic_1 == format!("{:?}", event721.signature()) && log.topics.len() == 4 {
                // keccak256(Transfer(address,address,uint256))

                let topic_2 = log.topics[1].clone().unwrap();
                let topic_3 = log.topics[2].clone().unwrap();
                let topic_4 = log.topics[3].clone().unwrap();

                let topic_2_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_2).unwrap();
                let topic_3_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_3).unwrap();
                let topic_4_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_4).unwrap();

                let from_address: String =
                    match ethabi::decode(&[ParamType::Address], topic_2_hash.as_bytes()) {
                        Ok(address) => {
                            if address.len() == 0 {
                                continue;
                            } else {
                                format!("{:?}", address[0].clone().into_address().unwrap())
                            }
                        }
                        Err(_) => continue,
                    };

                let to_address = match ethabi::decode(&[ParamType::Address], topic_3_hash.as_bytes()) {
                    Ok(address) => {
                        if address.len() == 0 {
                            continue;
                        } else {
                            format!("{:?}", address[0].clone().into_address().unwrap())
                        }
                    }
                    Err(_) => continue,
                };

                let token_id = match ethabi::decode(&[ParamType::Uint(256)], topic_4_hash.as_bytes()) {
                    Ok(id) => {
                        if id.len() == 0 {
                            continue;
                        } else {
                            BigDecimal::from_str(&format!("{:?}", id[0].clone().into_uint().unwrap())).unwrap()
                        }
                    }
                    Err(_) => continue,
                };

                let db_transfers = DatabaseNftTransfer {
                    hash: log.hash.clone(),
                    chain: log.chain.to_owned(),
                    log_index: log.log_index,
                    transfer_index: 0,
                    transfer_type: "ERC721Transfer".to_owned(),
                    token: log.address.clone(),
                    from_address,
                    to_address,
                    token_id,
                    value: BigDecimal::from_i64(1).unwrap(),
                    nft_tokens_parsed: false,
                    nft_balances_parsed: false,
                };
    
                db_nft_transfers.push(db_transfers)
            } else if topic_1 == format!("{:?}", event1155single.signature()) && log.topics.len() == 4 {
                // keccak256(TransferSingle(address,address,address,uint256,uint256))

                let topic_3 = log.topics[2].clone().unwrap();
                let topic_4 = log.topics[3].clone().unwrap();

                let topic_3_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_3).unwrap();
                let topic_4_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_4).unwrap();

                let data_bytes: Bytes =
                    array_bytes::hex_n_into::<String, Bytes, 32>(log.data.clone()).unwrap();

                let from_address: String =
                    match ethabi::decode(&[ParamType::Address], topic_3_hash.as_bytes()) {
                        Ok(address) => {
                            if address.len() == 0 {
                                continue;
                            } else {
                                format!("{:?}", address[0].clone().into_address().unwrap())
                            }
                        }
                        Err(_) => continue,
                    };
    
                let to_address = match ethabi::decode(&[ParamType::Address], topic_4_hash.as_bytes()) {
                    Ok(address) => {
                        if address.len() == 0 {
                            continue;
                        } else {
                            format!("{:?}", address[0].clone().into_address().unwrap())
                        }
                    }
                    Err(_) => continue,
                };

                let (token_id, value) = match ethabi::decode(
                    &[ParamType::Uint(256), ParamType::Uint(256)],
                    &data_bytes.0[..],
                ) {
                    Ok(value) => {
                        if value.len() < 2 {
                            continue;
                        } else {
                            (
                                BigDecimal::from_str(&format!("{:?}", value[0].clone().into_uint().unwrap())).unwrap(),
                                BigDecimal::from_str(&format!("{:?}", value[1].clone().into_uint().unwrap())).unwrap(),
                            )
                        }
                    }
                    Err(_) => continue,
                };

                let db_transfers = DatabaseNftTransfer {
                    hash: log.hash.clone(),
                    chain: log.chain.to_owned(),
                    log_index: log.log_index,
                    transfer_index: 0,
                    transfer_type: "ERC1155TransferSingle".to_owned(),
                    token: log.address.clone(),
                    from_address,
                    to_address,
                    token_id,
                    value,
                    nft_tokens_parsed: false,
                    nft_balances_parsed: false,
                };
    
                db_nft_transfers.push(db_transfers)
            } else if topic_1 == format!("{:?}", event1155batch.signature()) && log.topics.len() == 4 {
                // keccak256(TransferBatch(address,address,address,uint256[],uint256[]))

                let topic_3 = log.topics[2].clone().unwrap();
                let topic_4 = log.topics[3].clone().unwrap();

                let topic_3_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_3).unwrap();
                let topic_4_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_4).unwrap();

                let data_bytes: Bytes =
                    array_bytes::hex_n_into::<String, Bytes, 32>(log.data.clone()).unwrap();

                let from_address: String =
                    match ethabi::decode(&[ParamType::Address], topic_3_hash.as_bytes()) {
                        Ok(address) => {
                            if address.len() == 0 {
                                continue;
                            } else {
                                format!("{:?}", address[0].clone().into_address().unwrap())
                            }
                        }
                        Err(_) => continue,
                    };
    
                let to_address = match ethabi::decode(&[ParamType::Address], topic_4_hash.as_bytes()) {
                    Ok(address) => {
                        if address.len() == 0 {
                            continue;
                        } else {
                            format!("{:?}", address[0].clone().into_address().unwrap())
                        }
                    }
                    Err(_) => continue,
                };

                match ethabi::decode(
                    &[ParamType::Array(Box::new(ParamType::Uint(256))), ParamType::Array(Box::new(ParamType::Uint(256)))],
                    &data_bytes.0[..],
                ) {
                    Ok(value) => {
                        if value.len() < 2 {
                            continue
                        } else {
                            let token_ids: Vec<String> = value[0]
                                .clone()
                                .into_fixed_array()
                                .unwrap()
                                .iter()
                                .map(|x| format!("{:?}", x.clone().into_uint().unwrap()))
                                .collect();
        
                            let values: Vec<String> = value[1]
                                .clone()
                                .into_fixed_array()
                                .unwrap()
                                .iter()
                                .map(|x| format!("{:?}", x.clone().into_uint().unwrap()))
                                .collect();
        
                            for (i, (token_id, value)) in token_ids.iter().zip(values.iter()).enumerate()
                            {
                                let db_transfers = DatabaseNftTransfer {
                                    hash: log.hash.clone(),
                                    chain: log.chain.to_owned(),
                                    log_index: log.log_index,
                                    transfer_index: i as i64,
                                    transfer_type: "ERC1155TransferSingle".to_owned(),
                                    token: log.address.clone(),
                                    from_address: from_address.clone(),
                                    to_address: to_address.clone(),
                                    token_id: BigDecimal::from_str(token_id).unwrap(),
                                    value: BigDecimal::from_str(value).unwrap(),
                                    nft_tokens_parsed: false,
                                    nft_balances_parsed: false,
                                };

                                db_nft_transfers.push(db_transfers)
                            }
                        }
                    }
                    Err(_) => continue,
                };
            }
        }

        let mut connection = db.establish_connection();

        let chunks = get_chunks(
            db_nft_transfers.len(),
            DatabaseNftTransfer::field_count(),
        );

        for (start, end) in chunks {
            diesel::insert_into(nft_transfers::dsl::nft_transfers)
                .values(&db_nft_transfers[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store NFT transfers into database");
        }

        info!(
            "Inserted {} NFT transfers to the database.",
            db_nft_transfers.len()
        );

        let log_chunks = get_chunks(db_parsed_logs.len(), DatabaseLog::field_count());

        for (start, end) in log_chunks {
            diesel::insert_into(logs::dsl::logs)
                .values(&db_parsed_logs[start..end])
                .on_conflict((logs::hash, logs::log_index))
                .do_update()
                .set(logs::nft_transfers_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed logs into database");
        }

        Ok(())
    }
}
