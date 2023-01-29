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
            .limit(50000)
            .load::<DatabaseErc20Transfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseErc20Transfer>) -> Result<()> {
        let mut connection = db.establish_connection();

        let zero_address = format_address(H160::zero());

        let (senders, receivers): (Vec<(String, String, String)>, Vec<(String, String, String)>) =
            transfers
                .into_iter()
                .filter(|transfer| transfer.from_address != zero_address)
                .map(|transfer| {
                    (
                        (
                            transfer.token.clone(),
                            transfer.from_address.clone(),
                            transfer.chain.clone(),
                        ),
                        (
                            transfer.token.clone(),
                            transfer.from_address.clone(),
                            transfer.chain.clone(),
                        ),
                    )
                })
                .unzip();

        info!(
            "ERC20Tokens: updating balances for {} senders and {} receivers",
            senders.len(),
            receivers.len(),
        );

        let mut unique_balances: HashSet<(String, String, String)> = HashSet::new();

        for (token, address, chain) in senders {
            unique_balances.insert((token, address, chain));
        }

        for (token, address, chain) in receivers {
            unique_balances.insert((token, address, chain));
        }

        let balances_ids: Vec<(String, String, String)> = unique_balances.into_iter().collect();

        let stored_balances = self.get_current_balances(db, &balances_ids);

        let mut balances: HashMap<String, DatabaseErc20Balance> = HashMap::new();

        for balance in stored_balances {
            let id = format!("{}-{}-{}", balance.token, balance.address, balance.chain);
            balances.insert(id, balance);
        }

        info!("ERC20Tokens: fetched {} balances to update", balances.len(),);

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            let amount = I256::from_dec_str(&transfer.value).unwrap();

            if sender != format_address(H160::zero()) {
                let sender_id = format!("{}-{}-{}", transfer.token, sender, transfer.chain);

                let stored_balance = balances.get(&sender_id);

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

                balances.insert(sender_id, sender_balance);
            }

            let receiver = transfer.to_address.clone();

            if receiver != format_address(H160::zero()) {
                let receiver_id = format!("{}-{}-{}", transfer.token, receiver, transfer.chain);

                let stored_balance = balances.get(&receiver_id);

                let mut receiver_balance: DatabaseErc20Balance;

                if stored_balance.is_none() {
                    receiver_balance = DatabaseErc20Balance {
                        address: sender.clone(),
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

                balances.insert(receiver_id, receiver_balance);
            }
        }

        let new_balances = balances.values();

        let mut query =
            String::from("INSERT INTO erc20_balances (address, balance, chain, token) VALUES ");

        for balance in new_balances {
            let value = format!(
                " ('{}', '{}', '{}', '{}')",
                balance.address, balance.balance, balance.chain, balance.token
            );
            query.push_str(&value);
        }

        let conflict = "ON CONFLICT (address, token, chain) DO UPDATE";
        query.push_str(conflict);

        sql_query(query).execute(&mut connection).unwrap();

        /* diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
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
        balances: &Vec<(String, String, String)>,
    ) -> Vec<DatabaseErc20Balance> {
        let mut connection = db.establish_connection();

        let (first_address, first_token, first_chain) = balances.first().unwrap();
        let mut query = format!(
            "SELECT * FROM erc20_balances WHERE address = '{}' AND token = '{}' AND chain = '{}'",
            first_address, first_token, first_chain
        );

        for (i, (address, token, chain)) in balances.into_iter().enumerate() {
            if i > 0 {
                let condition = format!(
                    " OR address = '{}' AND token = '{}' AND chain = '{}'",
                    address, token, chain
                );
                query.push_str(&condition)
            }
        }

        let results: Vec<DatabaseErc20Balance> = sql_query(query)
            .load::<DatabaseErc20Balance>(&mut connection)
            .unwrap();

        return results;
    }
}
