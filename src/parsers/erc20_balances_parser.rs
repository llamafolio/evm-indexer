use crate::{
    db::{
        db::EVMDatabase,
        schema::{erc20_balances, erc20_tokens, erc20_transfers},
    },
    utils::format_address,
};
use anyhow::Result;
use diesel::{prelude::*, result::Error};
use ethers::{
    types::{H160, U256},
    utils::format_units,
};
use field_count::FieldCount;

use super::erc20_transfers_parser::DatabaseErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = erc20_balances)]
pub struct DatabaseErc20Balance {
    pub address: String,
    pub chain: String,
    pub token: String,
    pub balance: String,
}

pub struct ERC20BalancesParser {}

impl ERC20BalancesParser {
    pub fn fetch(&self, db: &EVMDatabase) -> Result<Vec<DatabaseErc20Transfer>> {
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

    pub async fn parse(
        &self,
        db: &EVMDatabase,
        transfers: &Vec<DatabaseErc20Transfer>,
    ) -> Result<()> {
        let mut connection = db.establish_connection();

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            let token_decimals: i64 = match erc20_tokens::table
                .select(erc20_tokens::decimals)
                .filter(erc20_tokens::address.eq(token.clone()))
                .first::<Option<i64>>(&mut connection)
            {
                Ok(decimals) => match decimals {
                    Some(decimals) => decimals,
                    None => continue,
                },
                Err(_) => continue,
            };

            let amount = format_units(
                U256::from_str_radix(&transfer.value, 10).unwrap(),
                token_decimals as usize,
            )
            .unwrap()
            .parse::<f64>()
            .unwrap();

            if sender != format_address(H160::zero()) {
                let mut sender_balance = match self.get_current_balance(
                    token.clone(),
                    sender.clone(),
                    transfer.chain.clone(),
                    db,
                ) {
                    Some(db_balance) => db_balance,
                    None => DatabaseErc20Balance {
                        address: sender,
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                        balance: "0".to_string(),
                    },
                };

                let balance: f64 = sender_balance.balance.parse::<f64>().unwrap();

                let final_balance = balance - amount;

                sender_balance.balance = final_balance.to_string();

                self.store_balance(&sender_balance, db).unwrap()
            }

            let receiver = transfer.to_address.clone();

            if receiver != format_address(H160::zero()) {
                // Add the balance to the receiver
                let mut receiver_balance = match self.get_current_balance(
                    token.clone(),
                    receiver.clone(),
                    transfer.chain.clone(),
                    db,
                ) {
                    Some(db_balance) => db_balance,
                    None => DatabaseErc20Balance {
                        address: receiver,
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                        balance: "0".to_string(),
                    },
                };

                let balance: f64 = receiver_balance.balance.parse::<f64>().unwrap();

                let final_balance = balance + amount;

                receiver_balance.balance = final_balance.to_string();

                self.store_balance(&receiver_balance, db).unwrap()
            }

            diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
                .values(transfer)
                .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
                .do_update()
                .set(erc20_transfers::erc20_balances_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed erc20 balances into database");
        }

        Ok(())
    }

    pub fn get_current_balance(
        &self,
        token: String,
        address: String,
        chain: String,
        db: &EVMDatabase,
    ) -> Option<DatabaseErc20Balance> {
        let mut connection = db.establish_connection();

        let db_balance: Result<DatabaseErc20Balance, Error> = erc20_balances::table
            .select(erc20_balances::all_columns)
            .filter(
                erc20_balances::token
                    .eq(token)
                    .and(erc20_balances::chain.eq(chain))
                    .and(erc20_balances::address.eq(address)),
            )
            .first::<DatabaseErc20Balance>(&mut connection);

        match db_balance {
            Ok(db_balance) => Some(db_balance),
            Err(_) => None,
        }
    }

    pub fn store_balance(&self, balance: &DatabaseErc20Balance, db: &EVMDatabase) -> Result<()> {
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
