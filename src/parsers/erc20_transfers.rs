use crate::db::{
    db::{get_chunks, Database},
    models::models::DatabaseLog,
};
use anyhow::Result;
use ethabi::{ethereum_types::H256, ParamType};
use ethers::types::Bytes;
use field_count::FieldCount;
use log::info;
use sqlx::QueryBuilder;

#[derive(Debug, Clone, FieldCount, sqlx::FromRow)]
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
    pub async fn fetch(&self, db: &Database) -> Result<Vec<DatabaseLog>> {
        let connection = db.get_connection();

        let rows = sqlx::query_as::<_, DatabaseLog>(
            "SELECT * FROM logs WHERE erc20_transfers_parsed = NULL OR erc20_transfers_parsed = false LIMIT 500",
        )
        .fetch_all(connection)
        .await;

        match rows {
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
                        indexed: true,
                    },
                    ethabi::EventParam {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
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

        let connection = db.get_connection();

        if db_erc20_transfers.len() > 0 {
            let chunks = get_chunks(
                db_erc20_transfers.len(),
                DatabaseErc20Transfer::field_count(),
            );

            for (start, end) in chunks {
                let mut query_builder =
            QueryBuilder::new("UPSERT INTO erc20_transfers (chain, erc20_balances_parsed, erc20_tokens_parsed, from_address, hash, log_index, to_address, token, value) ");

                query_builder.push_values(
                    &db_erc20_transfers[start..end],
                    |mut row, erc20_transfer| {
                        row.push_bind(erc20_transfer.chain.clone())
                            .push_bind(erc20_transfer.erc20_balances_parsed)
                            .push_bind(erc20_transfer.erc20_tokens_parsed)
                            .push_bind(erc20_transfer.from_address.clone())
                            .push_bind(erc20_transfer.hash.clone())
                            .push_bind(erc20_transfer.log_index.clone())
                            .push_bind(erc20_transfer.to_address.clone())
                            .push_bind(erc20_transfer.token.clone())
                            .push_bind(erc20_transfer.value.clone());
                    },
                );

                let query = query_builder.build();

                query
                    .execute(connection)
                    .await
                    .expect("Unable to store erc20 transfers into database");
            }
        }

        info!(
            "Inserted {} erc20 transfers to the database.",
            db_erc20_transfers.len()
        );

        if db_parsed_logs.len() > 0 {
            let chunks = get_chunks(db_parsed_logs.len(), DatabaseLog::field_count());

            for (start, end) in chunks {
                let mut query_builder = QueryBuilder::new("UPSERT INTO logs(address, chain, data, erc20_transfers_parsed, hash, log_index, removed, topics) ");

                query_builder.push_values(&db_parsed_logs[start..end], |mut row, log| {
                    row.push_bind(log.address.clone())
                        .push_bind(log.chain.clone())
                        .push_bind(log.data.clone())
                        .push_bind(log.erc20_transfers_parsed.clone())
                        .push_bind(log.hash.clone())
                        .push_bind(log.log_index.clone())
                        .push_bind(log.removed.clone())
                        .push_bind(log.topics.clone());
                });

                let query = query_builder.build();

                query
                    .execute(connection)
                    .await
                    .expect("Unable to update parsed logs into database");
            }
        }

        Ok(())
    }
}
