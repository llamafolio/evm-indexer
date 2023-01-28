use std::ops::{Add, Sub};

use crate::{
    db::{
        db::Database,
        schema::{erc20_balances, erc20_transfers},
    },
    utils::{format_address, format_singed_number},
};
use anyhow::Result;
use diesel::{prelude::*, result::Error};
use ethers::types::{H160, I256};
use field_count::FieldCount;
use jsonrpsee::tracing::info;

use super::erc20_transfers::DatabaseErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = erc20_balances)]
pub struct DatabaseErc20Balance {
    pub address: String,
    pub chain: String,
    pub token: String,
    pub balance: String,
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
            .limit(5000)
            .load::<DatabaseErc20Transfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseErc20Transfer>) -> Result<()> {
        let mut connection = db.establish_connection();

        let mut count_received = 0;
        let mut count_sent = 0;

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            let amount = I256::from_dec_str(&transfer.value).unwrap();

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

                let balance: I256 = match I256::from_dec_str(&sender_balance.balance) {
                    Ok(balance) => balance,
                    Err(_) => continue,
                };

                let final_balance = balance.sub(amount);

                sender_balance.balance = format_singed_number(final_balance);

                self.store_balance(&sender_balance, db).unwrap();

                count_sent += 1;
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

                let balance: I256 = I256::from_dec_str(&receiver_balance.balance).unwrap();

                let final_balance = balance.add(amount);

                receiver_balance.balance = format_singed_number(final_balance);

                self.store_balance(&receiver_balance, db).unwrap();

                count_received += 1;
            }

            diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
                .values(transfer)
                .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
                .do_update()
                .set(erc20_transfers::erc20_balances_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed erc20 balances into database");
        }

        info!(
            "Inserted {} sent balances and {} received balances.",
            count_sent, count_received
        );

        Ok(())
    }

    pub fn get_current_balance(
        &self,
        token: String,
        address: String,
        chain: String,
        db: &Database,
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
