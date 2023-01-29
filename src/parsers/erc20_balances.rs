use std::collections::{HashMap, HashSet};

use crate::{
    db::{
        db::Database,
        schema::{erc20_balances, erc20_tokens, erc20_transfers},
    },
    parsers::erc20_tokens::ERC20Tokens,
    utils::format_address,
};
use anyhow::Result;
use diesel::{prelude::*, result::Error, sql_query};
use ethers::{
    types::{H160, U256},
    utils::format_units,
};
use field_count::FieldCount;
use futures::future::join_all;
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
                    transfer.from_address.clone(),
                    transfer.token.clone(),
                    transfer.chain.clone(),
                )
            })
            .collect();

        let receivers: Vec<(String, String, String)> = transfers
            .into_iter()
            .filter(|transfer| transfer.to_address != zero_address)
            .map(|transfer| {
                (
                    transfer.to_address.clone(),
                    transfer.token.clone(),
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

        for (address, token, chain) in senders {
            unique_balances.insert((address, token, chain));
        }

        for (address, token, chain) in receivers {
            unique_balances.insert((address, token, chain));
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
                    balance.address.clone(),
                    balance.token.clone(),
                    balance.chain.clone(),
                ),
                balance,
            );
        }

        let mut parsed_transfers = vec![];

        let mut missing_tokens: HashSet<(String, String)> = HashSet::new();

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            let decimals = match tokens_map.get(&(transfer.token.clone(), transfer.chain.clone())) {
                Some(token_data) => match token_data.decimals {
                    Some(decimals) => decimals,
                    None => {
                        missing_tokens.insert((transfer.token.clone(), transfer.chain.clone()));
                        continue;
                    }
                },
                None => {
                    missing_tokens.insert((transfer.token.clone(), transfer.chain.clone()));
                    continue;
                }
            };

            let amount_value = U256::from_dec_str(&transfer.value).unwrap();

            let amount: f64 = format_units(amount_value, decimals as usize)
                .unwrap()
                .parse()
                .unwrap();

            if sender != format_address(H160::zero()) {
                let id = (
                    sender.clone(),
                    transfer.token.clone(),
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
                    receiver.clone(),
                    transfer.token.clone(),
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

        let total_new_balances = new_balances.len();

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

        if total_new_balances > 0 {
            sql_query(query).execute(&mut connection).unwrap();
        }

        if parsed_transfers.len() > 0 {
            diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
                .values(parsed_transfers)
                .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
                .do_update()
                .set(erc20_transfers::erc20_balances_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed erc20 balances into database");
        }

        info!("Inserted {} balances", total_new_balances);

        if missing_tokens.len() > 0 {
            let erc20_tokens = ERC20Tokens {};

            info!(
                "Fetching data for {} missing tokens data",
                missing_tokens.len()
            );

            let mut works = vec![];
            for (token, chain) in missing_tokens {
                let id = format!("{}-{}", token, chain);
                works.push(erc20_tokens.get_token_metadata(id))
            }

            let result = join_all(works).await;

            let tokens: Vec<DatabaseErc20Token> = result
                .into_iter()
                .filter(|token| token.is_some())
                .map(|token| token.unwrap())
                .collect();

            diesel::insert_into(erc20_tokens::dsl::erc20_tokens)
                .values(&tokens)
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .unwrap();

            info!("Inserted {} missing tokens data", tokens.len());
        }
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

        for (address, token, chain) in balances {
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
