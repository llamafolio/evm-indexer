use std::collections::{HashMap, HashSet};

use crate::{
    db::{
        db::{get_chunks, Database},
        schema::{nft_balances, nft_transfers},
    },
    parsers::nft_tokens::NftTokens,
    utils::format_address,
};
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::{prelude::*, result::Error, sql_query};
use ethers::{
    types::{H160, U256},
    utils::format_units,
};
use field_count::FieldCount;
use futures::future::join_all;
use jsonrpsee::tracing::info;
use redis::Commands;

use super::{nft_tokens::DatabaseNftToken, nft_transfers::DatabaseNftTransfer};

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount, QueryableByName)]
#[diesel(table_name = nft_balances)]
pub struct DatabaseNftBalance {
    pub address: String,
    pub chain: String,
    pub token: String,
    pub token_id: BigDecimal,
    pub balance: BigDecimal,
}

#[derive(Clone)]
pub struct NftBalances {}

impl NftBalances {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseNftTransfer>> {
        let mut connection = db.establish_connection();

        let transfers: Result<Vec<DatabaseNftTransfer>, Error> = nft_transfers::table
            .select(nft_transfers::all_columns)
            .filter(
                nft_transfers::nft_balances_parsed
                    .is_null()
                    .or(nft_transfers::nft_balances_parsed.eq(false)),
            )
            .limit(10000)
            .load::<DatabaseNftTransfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseNftTransfer>) -> Result<()> {
        let mut connection = db.establish_connection();
        let mut redis_connection = db.redis.get_connection().unwrap();

        let zero_address = format_address(H160::zero());

        let senders: Vec<(String, String, String, BigDecimal)> = transfers
            .into_iter()
            .filter(|transfer| transfer.from_address != zero_address)
            .map(|transfer| {
                (
                    transfer.from_address.clone(),
                    transfer.token.clone(),
                    transfer.chain.clone(),
                    transfer.token_id.clone(),
                )
            })
            .collect();

        let receivers: Vec<(String, String, String, BigDecimal)> = transfers
            .into_iter()
            .filter(|transfer| transfer.to_address != zero_address)
            .map(|transfer| {
                (
                    transfer.to_address.clone(),
                    transfer.token.clone(),
                    transfer.chain.clone(),
                    transfer.token_id.clone(),
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
            "NFTTokens: updating balances for {} senders and {} receivers from {} total tokens {} tokens with data",
            senders.len(),
            receivers.len(),
            tokens.len(),
            tokens_data.len()
        );

        let mut tokens_map: HashMap<(String, String), DatabaseNftToken> = HashMap::new();

        for token in tokens_data {
            tokens_map.insert((token.address.clone(), token.chain.clone()), token);
        }

        let mut unique_balances: HashSet<(String, String, String, BigDecimal)> = HashSet::new();

        for (address, token, chain, token_id) in senders {
            unique_balances.insert((address, token, chain, token_id));
        }

        for (address, token, chain, token_id) in receivers {
            unique_balances.insert((address, token, chain, token_id));
        }

        let balances_ids: Vec<(String, String, String, BigDecimal)> = unique_balances.into_iter().collect();

        let stored_balances = self.get_current_balances(db, &balances_ids);

        info!(
            "ERC20Tokens: fetched {} balances to update",
            stored_balances.len()
        );

        let mut balances: HashMap<(String, String, String, BigDecimal), DatabaseNftBalance> = HashMap::new();

        for balance in stored_balances {
            balances.insert(
                (
                    balance.address.clone(),
                    balance.token.clone(),
                    balance.chain.clone(),
                    balance.token_id.clone(),
                ),
                balance,
            );
        }

        let mut parsed_transfers = vec![];

        // let mut retried_transfers_passed = vec![];

        // let mut missing_tokens: HashSet<(String, String, BigDecimal)> = HashSet::new();

        for transfer in transfers {
            let token = transfer.token.clone();

            let sender = transfer.from_address.clone();

            // let redis_retries_key =
            //     format!("BALANCE_PARSE-{}-{}-{}", token.clone(), transfer.chain.clone(), transfer.token_id.clone());

            // let retries = match redis_connection.get::<String, i64>(redis_retries_key.clone()) {
            //     Ok(retries) => retries,
            //     Err(_) => 0,
            // };

            // if retries > 5 {
            //     retried_transfers_passed.push(transfer.to_owned());
            //     continue;
            // }

            // let decimals = match tokens_map.get(&(transfer.token.clone(), transfer.chain.clone())) {
            //     Some(token_data) => match token_data.decimals {
            //         Some(decimals) => decimals,
            //         None => {
            //             missing_tokens.insert((transfer.token.clone(), transfer.chain.clone()));
            //             let new_retries = retries + 1;

            //             let _: () = redis_connection
            //                 .set(redis_retries_key.clone(), new_retries)
            //                 .unwrap();

            //             continue;
            //         }
            //     },
            //     None => {
            //         missing_tokens.insert((transfer.token.clone(), transfer.chain.clone()));

            //         let new_retries = retries + 1;

            //         let _: () = redis_connection
            //             .set(redis_retries_key.clone(), new_retries)
            //             .unwrap();

            //         continue;
            //     }
            // };

            // let amount_value = U256::from_dec_str(&transfer.value).unwrap();

            // let amount: f64 = match format_units(amount_value, decimals as usize) {
            //     Ok(amount) => match amount.parse::<f64>() {
            //         Ok(amount) => amount,
            //         Err(_) => {
            //             let new_retries = retries + 1;

            //             let _: () = redis_connection
            //                 .set(redis_retries_key.clone(), new_retries)
            //                 .unwrap();

            //             continue;
            //         }
            //     },
            //     Err(_) => {
            //         let new_retries = retries + 1;

            //         let _: () = redis_connection
            //             .set(redis_retries_key.clone(), new_retries)
            //             .unwrap();

            //         continue;
            //     }
            // };

            if sender != format_address(H160::zero()) {
                let id = (
                    sender.clone(),
                    transfer.token.clone(),
                    transfer.chain.clone(),
                    transfer.token_id.clone(),
                );

                let stored_balance = balances.get(&id);

                let mut sender_balance: DatabaseNftBalance;

                if stored_balance.is_none() {
                    sender_balance = DatabaseNftBalance {
                        address: sender.clone(),
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                        token_id: transfer.token_id.clone(),
                        balance: BigDecimal::from_i8(0).unwrap(),
                    };
                } else {
                    sender_balance = stored_balance.unwrap().to_owned();
                }

                sender_balance.balance = sender_balance.balance - transfer.value.clone();

                balances.insert(id, sender_balance);
            }

            let receiver = transfer.to_address.clone();

            if receiver != format_address(H160::zero()) {
                let id = (
                    receiver.clone(),
                    transfer.token.clone(),
                    transfer.chain.clone(),
                    transfer.token_id.clone(),
                );

                let stored_balance = balances.get(&id);

                let mut receiver_balance: DatabaseNftBalance;

                if stored_balance.is_none() {
                    receiver_balance = DatabaseNftBalance {
                        address: receiver.clone(),
                        chain: transfer.chain.clone(),
                        token: token.clone(),
                        token_id: transfer.token_id.clone(),
                        balance: BigDecimal::from_i8(0).unwrap(),
                    };
                } else {
                    receiver_balance = stored_balance.unwrap().to_owned();
                }

                receiver_balance.balance = receiver_balance.balance + transfer.value.clone();

                balances.insert(id, receiver_balance);
            }

            parsed_transfers.push(transfer.to_owned());
        }

        let new_balances = balances.values();

        let total_new_balances = new_balances.len();

        let mut query =
            String::from("UPSERT INTO nft_balances (address, chain, token, token_id, balance) VALUES");

        for balance in new_balances {
            let value = format!(
                " ('{}', '{}', '{}', {}, {}),",
                balance.address, balance.chain, balance.token, balance.token_id, balance.balance
            );
            query.push_str(&value);
        }

        // Remove the last comma of the value list.
        query.pop();

        if total_new_balances > 0 {
            sql_query(query).execute(&mut connection).unwrap();
        }

        let mut total_transfers_parsed: Vec<DatabaseNftTransfer> = Vec::new();

        total_transfers_parsed.append(&mut parsed_transfers);
        // total_transfers_parsed.append(&mut retried_transfers_passed);

        if total_transfers_parsed.len() > 0 {
            let chunks = get_chunks(
                total_transfers_parsed.len(),
                DatabaseNftTransfer::field_count(),
            );

            for (start, end) in chunks {
                diesel::insert_into(nft_transfers::dsl::nft_transfers)
                    .values(&total_transfers_parsed[start..end])
                    .on_conflict((nft_transfers::hash, nft_transfers::log_index))
                    .do_update()
                    .set(nft_transfers::nft_balances_parsed.eq(true))
                    .execute(&mut connection)
                    .expect("Unable to update parsed nft balances into database");
            }
        }

        info!(
            "Inserted {} nft balances",
            total_new_balances,
            // retried_transfers_passed.len()
        );

        // if missing_tokens.len() > 0 {
        //     let erc20_tokens = ERC20Tokens {};

        //     info!(
        //         "Fetching data for {} missing tokens data",
        //         missing_tokens.len()
        //     );

        //     let mut works = vec![];
        //     for (token, chain) in missing_tokens {
        //         let id = format!("{}-{}", token, chain);
        //         works.push(erc20_tokens.get_token_metadata(id))
        //     }

        //     let result = join_all(works).await;

        //     let tokens: Vec<DatabaseNftToken> = result
        //         .into_iter()
        //         .filter(|token| token.is_some())
        //         .map(|token| token.unwrap())
        //         .collect();

        //     let tokens_amount = tokens.len();

        //     let mut query = String::from(
        //         "UPSERT INTO nft_tokens (address, chain, nft_type, name, symbol, contract_uri) VALUES",
        //     );

        //     for token in tokens {
        //         let name = match token.name {
        //             Some(name) => {
        //                 let name_fixed: String = name.replace("'", "");

        //                 let name_bytes = name_fixed.as_bytes();

        //                 let name_parsed = String::from_utf8_lossy(name_bytes);

        //                 format!("'{}'", name_parsed)
        //             }
        //             None => String::from("NULL"),
        //         };

        //         let symbol = match token.symbol {
        //             Some(symbol) => {
        //                 let symbol_fixed: String = symbol.replace("'", "");

        //                 let symbol_bytes = symbol_fixed.as_bytes();

        //                 let symbol_parsed = String::from_utf8_lossy(symbol_bytes);

        //                 format!("'{}'", symbol_parsed)
        //             }
        //             None => String::from("NULL"),
        //         };

        //         // TODO: Must find a bettwe way to encode SQL. It's too danger to do this way
        //         let contract_uri = match token.contract_uri {
        //             Some(contract_uri) => {
        //                 let contract_uri_fixed: String = contract_uri.replace("'", "");

        //                 let contract_uri_bytes = contract_uri_fixed.as_bytes();

        //                 let contract_uri_parsed = String::from_utf8_lossy(contract_uri_bytes);

        //                 format!("'{}'", contract_uri_parsed)
        //             }
        //             None => String::from(""),
        //         };

        //         let value = format!(
        //             " ('{}', '{}', '{}', {}, {}, {}),",
        //             token.address,
        //             token.chain,
        //             token.nft_type,
        //             name,
        //             symbol,
        //             contract_uri
        //         );

        //         query.push_str(&value);
        //     }

        //     query.pop();

        //     if tokens_amount > 0 {
        //         sql_query(query).execute(&mut connection).unwrap();
        //     }

        //     info!("Inserted {} missing tokens data", tokens_amount);
        // }

        Ok(())
    }

    pub fn get_current_balances(
        self: &NftBalances,
        db: &Database,
        balances: &Vec<(String, String, String, BigDecimal)>,
    ) -> Vec<DatabaseNftBalance> {
        let mut connection = db.establish_connection();

        let mut query =
            String::from("SELECT * FROM nft_balances WHERE (address, token, chain, token_id) IN ( VALUES");

        for (address, token, chain, token_id) in balances {
            let condition = format!("(('{}','{}','{}',{})),", address, token, chain, token_id);
            query.push_str(&condition)
        }

        query.pop();
        query.push_str(")");

        let results: Vec<DatabaseNftBalance> = sql_query(query)
            .load::<DatabaseNftBalance>(&mut connection)
            .unwrap();

        return results;
    }

    pub fn get_tokens(
        self: &NftBalances,
        db: &Database,
        tokens: &Vec<(String, String)>,
    ) -> Vec<DatabaseNftToken> {
        let mut connection = db.establish_connection();

        let mut query =
            String::from("SELECT * FROM nft_tokens WHERE (address, chain) IN ( VALUES ");

        for (token, chain) in tokens {
            let condition = format!("(('{}','{}')),", token, chain);
            query.push_str(&condition)
        }

        query.pop();
        query.push_str(")");

        let results: Vec<DatabaseNftToken> = sql_query(query)
            .load::<DatabaseNftToken>(&mut connection)
            .unwrap();

        return results;
    }
}
