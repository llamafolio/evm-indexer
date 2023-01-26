use crate::db::{
    db::{get_chunks, Database},
    models::models::DatabaseLog,
    schema::{erc20_transfers, logs},
};
use anyhow::Result;
use diesel::{prelude::*, result::Error};
use ethabi::{ethereum_types::H256, ParamType};
use ethers::types::Bytes;
use field_count::FieldCount;
use log::info;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = erc20_transfers)]
pub struct DatabaseErc20Transfer {
    pub chain: String,
    pub erc20_balances_parsed: bool,
    pub erc20_tokens_parsed: bool,
    pub from_address: String,
    pub hash: String,
    pub log_index: i64,
    pub to_address: String,
    pub token: String,
    pub value: String,
}

pub struct ERC20Transfers {}

impl ERC20Transfers {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseLog>> {
        let mut connection = db.establish_connection();

        let logs: Result<Vec<DatabaseLog>, Error> = logs::table
            .select(logs::all_columns)
            .filter(
                logs::erc20_transfers_parsed
                    .is_null()
                    .or(logs::erc20_transfers_parsed.eq(false)),
            )
            .limit(50000)
            .load::<DatabaseLog>(&mut connection);

        match logs {
            Ok(logs) => Ok(logs),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, logs: &Vec<DatabaseLog>) -> Result<()> {
        let mut db_erc20_transfers = Vec::new();

        let mut db_parsed_logs = Vec::new();

        for log in logs {
            let mut parsed_log = log.to_owned();

            parsed_log.erc20_transfers_parsed = true;

            db_parsed_logs.push(parsed_log);

            if log.topics.len() != 3 {
                continue;
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

            let topic_1 = log.topics[0].clone().unwrap();

            // Check the first topic against keccak256(Transfer(address,address,uint256))
            if topic_1 != format!("{:?}", event.signature()) {
                continue;
            }

            let topic_2 = log.topics[1].clone().unwrap();
            let topic_3 = log.topics[2].clone().unwrap();

            let topic_2_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_2).unwrap();

            let topic_3_hash: H256 = array_bytes::hex_n_into::<String, H256, 32>(topic_3).unwrap();

            let data_bytes: Bytes =
                array_bytes::hex_n_into::<String, Bytes, 32>(log.data.clone()).unwrap();

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

            let value = match ethabi::decode(&[ParamType::Uint(256)], &data_bytes.0[..]) {
                Ok(value) => {
                    if value.len() == 0 {
                        continue;
                    } else {
                        format!("{:?}", value[0].clone().into_uint().unwrap())
                    }
                }
                Err(_) => continue,
            };

            let db_transfers = DatabaseErc20Transfer {
                hash: log.hash.clone(),
                chain: log.chain.to_owned(),
                log_index: log.log_index,
                token: log.address.clone(),
                from_address,
                to_address,
                value,
                erc20_tokens_parsed: false,
                erc20_balances_parsed: false,
            };

            db_erc20_transfers.push(db_transfers)
        }

        let mut connection = db.establish_connection();

        let chunks = get_chunks(
            db_erc20_transfers.len(),
            DatabaseErc20Transfer::field_count(),
        );

        for (start, end) in chunks {
            diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
                .values(&db_erc20_transfers[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store erc20 transfers into database");
        }

        info!(
            "Inserted {} erc20 transfers to the database.",
            db_erc20_transfers.len()
        );

        let log_chunks = get_chunks(db_parsed_logs.len(), DatabaseLog::field_count());

        for (start, end) in log_chunks {
            diesel::insert_into(logs::dsl::logs)
                .values(&db_parsed_logs[start..end])
                .on_conflict((logs::hash, logs::log_index))
                .do_update()
                .set(logs::erc20_transfers_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed logs into database");
        }

        Ok(())
    }
}
