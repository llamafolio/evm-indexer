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
use diesel::{prelude::*, result::Error};
use ethers::types::{H160, I256};
use field_count::FieldCount;
use futures::future::join_all;
use jsonrpsee::tracing::info;

use super::erc20_transfers::DatabaseErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = erc20_balances)]
pub struct DatabaseErc20Balance {
    pub address: String,
    pub balance: String,
    pub chain: String,
    pub token: String,
}

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
            .limit(500)
            .load::<DatabaseErc20Transfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseErc20Transfer>) -> Result<()> {
        let mut connection = db.establish_connection();

        let zero_address = format_address(H160::zero());

        let senders: Vec<(String, String, String)> = transfers
            .into_iter()
            .filter(|transfer| transfer.from_address != zero_address)
            .map(|transfer| {
                (
                    transfer.token.clone(),
                    transfer.from_address.clone(),
                    transfer.chain.clone(),
                )
            })
            .collect();

        let receivers: Vec<(String, String, String)> = transfers
            .into_iter()
            .filter(|transfer| transfer.to_address != zero_address)
            .map(|transfer| {
                (
                    transfer.token.clone(),
                    transfer.to_address.clone(),
                    transfer.chain.clone(),
                )
            })
            .collect();

        info!(
            "ERC20Tokens: updating balances for {} senders and {} receivers",
            senders.len(),
            receivers.len()
        );

        let mut balances: HashMap<String, DatabaseErc20Balance> = HashMap::new();

        let mut unique_values: HashSet<String> = HashSet::new();

        for (token, address, chain) in senders {
            unique_values.insert(format!("{}-{}-{}", token, address, chain));
        }

        for (token, address, chain) in receivers {
            unique_values.insert(format!("{}-{}-{}", token, address, chain));
        }

        println!("{}", unique_values.len());

        let mut fetch_balances = vec![];

        for value in unique_values {
            let data: Vec<&str> = value.split("-").collect();

            fetch_balances.push(self.get_current_balance(
                data[0].to_owned(),
                data[1].to_owned(),
                data[2].to_owned(),
                db,
            ))
        }
        println!("{}", fetch_balances.len());

        let balances = join_all(fetch_balances).await;

        println!("{}", balances.len());

        /* for transfer in transfers {
                    let token = transfer.token.clone();

                    let sender = transfer.from_address.clone();

                    let amount = I256::from_dec_str(&transfer.value).unwrap();

                    if sender != format_address(H160::zero()) {
                        let stored_balance = balances_changes.get(&sender);

                        let mut sender_balance: DatabaseErc20Balance;

                        if stored_balance.is_some() {
                            sender_balance = stored_balance.unwrap().to_owned();
                        } else {
                            sender_balance = match self.get_current_balance(
                                token.clone(),
                                sender.clone(),
                                transfer.chain.clone(),
                                db,
                            ) {
                                Some(db_balance) => db_balance,
                                None => DatabaseErc20Balance {
                                    address: sender.clone(),
                                    balance: "0".to_string(),
                                    chain: transfer.chain.clone(),
                                    token: token.clone(),
                                },
                            };
                        }

                        let balance: I256 = I256::from_dec_str(&sender_balance.balance).unwrap();

                        let final_balance = balance.add(amount);

                        sender_balance.balance = final_balance.to_string();

                        balances_changes.insert(sender.clone(), sender_balance);

                        count_sent += 1;
                    }

                    let receiver = transfer.to_address.clone();

                    if receiver != format_address(H160::zero()) {
                        let stored_balance = balances_changes.get(&receiver);

                        let mut receiver_balance: DatabaseErc20Balance;

                        if stored_balance.is_some() {
                            receiver_balance = stored_balance.unwrap().to_owned();
                        } else {
                            receiver_balance = match self.get_current_balance(
                                token.clone(),
                                receiver.clone(),
                                transfer.chain.clone(),
                                db,
                            ) {
                                Some(db_balance) => db_balance,
                                None => DatabaseErc20Balance {
                                    address: receiver.clone(),
                                    balance: "0".to_string(),
                                    chain: transfer.chain.clone(),
                                    token: token.clone(),
                                },
                            };
                        }

                        let balance: I256 = I256::from_dec_str(&receiver_balance.balance).unwrap();

                        let final_balance = balance.sub(amount);

                        receiver_balance.balance = final_balance.to_string();

                        balances_changes.insert(receiver.clone(), receiver_balance);

                        count_received += 1;
                    }
                }

                let updates: Vec<&DatabaseErc20Balance> = balances_changes.values().collect();
                println!("{} balances to store", updates.len());

                /* diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
                .values(transfers)
                .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
                .do_update()
                .set(erc20_transfers::erc20_balances_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed erc20 balances into database"); */

                info!(
                    "Inserted {} sent balances and {} received balances.",
                    count_sent, count_received
                );
        */
        Ok(())
    }

    pub async fn get_current_balance(
        &self,
        token: String,
        address: String,
        chain: String,
        db: &Database,
    ) -> DatabaseErc20Balance {
        let mut connection = db.establish_connection();

        let db_balance: Result<DatabaseErc20Balance, Error> = erc20_balances::table
            .select(erc20_balances::all_columns)
            .filter(
                erc20_balances::address
                    .eq(address.clone())
                    .and(erc20_balances::token.eq(token.clone()))
                    .and(erc20_balances::chain.eq(chain.clone())),
            )
            .first::<DatabaseErc20Balance>(&mut connection);

        let balance = match db_balance {
            Ok(db_balance) => db_balance,
            Err(_) => {
                return DatabaseErc20Balance {
                    address: address.clone(),
                    balance: "0".to_string(),
                    chain: chain.clone(),
                    token: token.clone(),
                }
            }
        };

        return balance;
    }

    pub fn store_balance(&self, balance: &DatabaseErc20Balance, db: &Database) -> Result<()> {
        let mut connection = db.establish_connection();

        diesel::insert_into(erc20_balances::dsl::erc20_balances)
            .values(balance)
            .on_conflict((
                erc20_balances::address,
                erc20_balances::token,
                erc20_balances::chain,
            ))
            .do_update()
            .set(erc20_balances::balance.eq(balance.balance.clone()))
            .execute(&mut connection)
            .expect("Unable to store erc20 balance");

        Ok(())
    }
}
