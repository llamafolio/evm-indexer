use std::{
    collections::{HashMap, HashSet},
    ops::{Add, Sub},
};

use crate::{
    db::{
        db::Database,
        schema::{erc20_balances, erc20_transfers},
    },
    utils::format_address,
};
use anyhow::Result;
use diesel::{prelude::*, result::Error, sql_query};
use ethers::types::{H160, I256};
use field_count::FieldCount;
use jsonrpsee::tracing::info;

use super::erc20_transfers::DatabaseErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount, QueryableByName)]
#[diesel(table_name = erc20_balances)]
pub struct DatabaseErc20Balance {
    pub address: String,
    pub balance: String,
    pub chain: String,
    pub token: String,
}

#[derive(Clone)]
pub struct ERC20Balances {}

impl ERC20Balances {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseErc20Transfer>> {
        let mut connection = db.establish_connection();

        let transfers: Result<Vec<DatabaseErc20Transfer>, Error> = erc20_transfers::table
            .select(erc20_transfers::all_columns)
            .filter(
                erc20_transfers::erc20_balances_parsed
                    .is_null()
                    .or(erc20_transfers::erc20_balances_parsed.eq(false)),
            )
            .limit(100)
            .load::<DatabaseErc20Transfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseErc20Transfer>) -> Result<()> {
        let mut connection = db.establish_connection();

        let zero_address = format_address(H160::zero());

        let senders: Vec<String> = transfers
            .into_iter()
            .filter(|transfer| transfer.from_address != zero_address)
            .map(|transfer| {
                format!(
                    "{}-{}-{}",
                    transfer.token.clone(),
                    transfer.from_address.clone(),
                    transfer.chain.clone(),
                )
            })
            .collect();

        let receivers: Vec<String> = transfers
            .into_iter()
            .filter(|transfer| transfer.to_address != zero_address)
            .map(|transfer| {
                format!(
                    "{}-{}-{}",
                    transfer.token.clone(),
                    transfer.to_address.clone(),
                    transfer.chain.clone(),
                )
            })
            .collect();

        info!(
            "ERC20Tokens: updating balances for {} senders and {} receivers",
            senders.len(),
            receivers.len(),
        );

        let mut unique_balances: HashSet<String> = HashSet::new();

        for balance_id in senders {
            unique_balances.insert(balance_id);
        }

        for balance_id in receivers {
            unique_balances.insert(balance_id);
        }

        let balances_ids: Vec<String> = unique_balances.into_iter().collect();

        let stored_balances = self.get_current_balances(db, &balances_ids);

        info!(
            "ERC20Tokens: fetched {} balances to update",
            stored_balances.len()
        );

        let mut balances: HashMap<String, DatabaseErc20Balance> = HashMap::new();

        for balance in stored_balances {
            balances.insert(
                format!(
                    "{}-{}-{}",
                    balance.token.clone(),
                    balance.address.clone(),
                    balance.chain.clone(),
                ),
                balance,
            );
        }

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            let amount = I256::from_dec_str(&transfer.value).unwrap();

            if sender != format_address(H160::zero()) {
                let id = format!(
                    "{}-{}-{}",
                    transfer.token.clone(),
                    sender.clone(),
                    transfer.chain.clone(),
                );

                let stored_balance = balances.get(&id);

                let mut sender_balance: DatabaseErc20Balance;

                if stored_balance.is_none() {
                    sender_balance = DatabaseErc20Balance {
                        address: sender.clone(),
                        balance: "0".to_string(),
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                    };
                } else {
                    sender_balance = stored_balance.unwrap().to_owned();
                }

                let new_balance = I256::from_dec_str(&sender_balance.balance)
                    .unwrap()
                    .sub(amount);

                sender_balance.balance = new_balance.to_string();

                balances.insert(id, sender_balance);
            }

            let receiver = transfer.to_address.clone();

            if receiver != format_address(H160::zero()) {
                let id = format!(
                    "{}-{}-{}",
                    transfer.token.clone(),
                    receiver.clone(),
                    transfer.chain.clone(),
                );

                let stored_balance = balances.get(&id);

                let mut receiver_balance: DatabaseErc20Balance;

                if stored_balance.is_none() {
                    receiver_balance = DatabaseErc20Balance {
                        address: receiver.clone(),
                        balance: "0".to_string(),
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                    };
                } else {
                    receiver_balance = stored_balance.unwrap().to_owned();
                }

                let new_balance = I256::from_dec_str(&receiver_balance.balance)
                    .unwrap()
                    .add(amount);

                receiver_balance.balance = new_balance.to_string();

                balances.insert(id, receiver_balance);
            }
        }

        let new_balances = balances.values();

        let mut query = String::from(
            "UPSERT INTO erc20_balances (balance_id, address, balance, chain, token) VALUES",
        );

        for balance in new_balances {
            let value = format!(
                " ('{}', '{}', '{}', '{}', '{}'),",
                format!(
                    "{}-{}-{}",
                    balance.token.clone(),
                    balance.address.clone(),
                    balance.chain.clone()
                ),
                balance.address,
                balance.balance,
                balance.chain,
                balance.token
            );
            query.push_str(&value);
        }

        // Remove the last comma of the value list.
        query.pop();

        sql_query(query).execute(&mut connection).unwrap();

        /*    diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
        .values(transfers)
        .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
        .do_update()
        .set(erc20_transfers::erc20_balances_parsed.eq(true))
        .execute(&mut connection)
        .expect("Unable to update parsed erc20 balances into database"); */

        info!("Inserted {} balances", balances.len());

        Ok(())
    }

    pub fn get_current_balances(
        self: &ERC20Balances,
        db: &Database,
        balances: &Vec<String>,
    ) -> Vec<DatabaseErc20Balance> {
        let mut connection = db.establish_connection();

        let mut query = format!(
            "SELECT * FROM erc20_balances WHERE balance_id = '{}'",
            balances.first().unwrap()
        );

        for (i, balance_id) in balances.into_iter().enumerate() {
            if i > 0 {
                let condition = format!(" OR balance_id = '{}'", balance_id);
                query.push_str(&condition)
            }
        }

        let results: Vec<DatabaseErc20Balance> = sql_query(query)
            .load::<DatabaseErc20Balance>(&mut connection)
            .unwrap();

        return results;
    }
}
