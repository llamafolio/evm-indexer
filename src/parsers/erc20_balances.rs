use std::collections::{HashMap, HashSet};

use crate::{
    db::{
        db::Database,
        schema::{erc20_balances, erc20_transfers},
    },
    utils::format_address,
};
use anyhow::Result;
use diesel::{prelude::*, result::Error, sql_query};
use ethers::{
    types::{H160, U256},
    utils::format_units,
};
use field_count::FieldCount;
use jsonrpsee::tracing::info;

use super::{erc20_tokens::DatabaseErc20Token, erc20_transfers::DatabaseErc20Transfer};

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount, QueryableByName)]
#[diesel(table_name = erc20_balances)]
pub struct DatabaseErc20Balance {
    pub address: String,
    pub balance: f64,
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
            .limit(5000)
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

        let tokens: Vec<(String, String)> = transfers
            .into_iter()
            .map(|transfer| (transfer.token.clone(), transfer.chain.clone()))
            .collect::<HashSet<(String, String)>>()
            .into_iter()
            .collect();

        let tokens_data = self.get_tokens(db, &tokens);

        info!(
            "ERC20Tokens: updating balances for {} senders and {} receivers from {} tokens",
            senders.len(),
            receivers.len(),
            tokens_data.len()
        );

        let mut tokens_map: HashMap<(String, String), DatabaseErc20Token> = HashMap::new();

        for token in tokens_data {
            tokens_map.insert((token.address.clone(), token.chain.clone()), token);
        }

        let mut unique_balances: HashSet<(String, String, String)> = HashSet::new();

        for (token, address, chain) in senders {
            unique_balances.insert((token, address, chain));
        }

        for (token, address, chain) in receivers {
            unique_balances.insert((token, address, chain));
        }

        let balances_ids: Vec<(String, String, String)> = unique_balances.into_iter().collect();

        let stored_balances = self.get_current_balances(db, &balances_ids);

        info!(
            "ERC20Tokens: fetched {} balances to update",
            stored_balances.len()
        );

        let mut balances: HashMap<(String, String, String), DatabaseErc20Balance> = HashMap::new();

        for balance in stored_balances {
            balances.insert(
                (
                    balance.token.clone(),
                    balance.address.clone(),
                    balance.chain.clone(),
                ),
                balance,
            );
        }

        let mut parsed_transfers = vec![];

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            let decimals = match tokens_map.get(&(transfer.token.clone(), transfer.chain.clone())) {
                Some(token_data) => match token_data.decimals {
                    Some(decimals) => decimals,
                    None => continue,
                },
                None => continue,
            };

            let amount: f64 = format_units(
                U256::from_dec_str(&transfer.value).unwrap(),
                decimals as usize,
            )
            .unwrap()
            .parse()
            .unwrap();

            if sender != format_address(H160::zero()) {
                let id = (
                    transfer.token.clone(),
                    sender.clone(),
                    transfer.chain.clone(),
                );

                let stored_balance = balances.get(&id);

                let mut sender_balance: DatabaseErc20Balance;

                if stored_balance.is_none() {
                    sender_balance = DatabaseErc20Balance {
                        address: sender.clone(),
                        balance: 0.0,
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                    };
                } else {
                    sender_balance = stored_balance.unwrap().to_owned();
                }

                sender_balance.balance = sender_balance.balance - amount;

                balances.insert(id, sender_balance);
            }

            let receiver = transfer.to_address.clone();

            if receiver != format_address(H160::zero()) {
                let id = (
                    transfer.token.clone(),
                    receiver.clone(),
                    transfer.chain.clone(),
                );

                let stored_balance = balances.get(&id);

                let mut receiver_balance: DatabaseErc20Balance;

                if stored_balance.is_none() {
                    receiver_balance = DatabaseErc20Balance {
                        address: receiver.clone(),
                        balance: 0.0,
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                    };
                } else {
                    receiver_balance = stored_balance.unwrap().to_owned();
                }

                receiver_balance.balance = receiver_balance.balance + amount;

                balances.insert(id, receiver_balance);
            }

            parsed_transfers.push(transfer);
        }

        let new_balances = balances.values();

        let mut query =
            String::from("UPSERT INTO erc20_balances (address, balance, chain, token) VALUES");

        for balance in new_balances {
            let value = format!(
                " ('{}', '{}', '{}', '{}'),",
                balance.address, balance.balance, balance.chain, balance.token
            );
            query.push_str(&value);
        }

        // Remove the last comma of the value list.
        query.pop();

        sql_query(query).execute(&mut connection).unwrap();

        diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
            .values(parsed_transfers)
            .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
            .do_update()
            .set(erc20_transfers::erc20_balances_parsed.eq(true))
            .execute(&mut connection)
            .expect("Unable to update parsed erc20 balances into database");

        info!("Inserted {} balances", balances.len());

        Ok(())
    }

    pub fn get_current_balances(
        self: &ERC20Balances,
        db: &Database,
        balances: &Vec<(String, String, String)>,
    ) -> Vec<DatabaseErc20Balance> {
        let mut connection = db.establish_connection();

        let mut query =
            String::from("SELECT * FROM erc20_balances WHERE (address, token, chain) IN ( VALUES");

        for (token, address, chain) in balances {
            let condition = format!("(('{}','{}','{}')),", address, token, chain);
            query.push_str(&condition)
        }

        query.pop();
        query.push_str(")");

        let results: Vec<DatabaseErc20Balance> = sql_query(query)
            .load::<DatabaseErc20Balance>(&mut connection)
            .unwrap();

        return results;
    }

    pub fn get_tokens(
        self: &ERC20Balances,
        db: &Database,
        tokens: &Vec<(String, String)>,
    ) -> Vec<DatabaseErc20Token> {
        let mut connection = db.establish_connection();

        let mut query =
            String::from("SELECT * FROM erc20_tokens WHERE (address, chain) IN ( VALUES ");

        for (token, chain) in tokens {
            let condition = format!("(('{}','{}')),", token, chain);
            query.push_str(&condition)
        }

        query.pop();
        query.push_str(")");

        let results: Vec<DatabaseErc20Token> = sql_query(query)
            .load::<DatabaseErc20Token>(&mut connection)
            .unwrap();

        return results;
    }
}
