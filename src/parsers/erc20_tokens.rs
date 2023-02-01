use std::{collections::HashSet, sync::Arc};

use crate::{
    chains::chains::{get_chain, get_chains},
    db::{
        db::{get_chunks, Database},
        schema::{erc20_tokens, erc20_transfers},
    },
};
use anyhow::Result;
use diesel::{prelude::*, result::Error, sql_query};
use ethabi::Address;
use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
};
use field_count::FieldCount;
use futures::future::join_all;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::erc20_transfers::DatabaseErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount, QueryableByName)]
#[diesel(table_name = erc20_tokens)]
pub struct DatabaseErc20Token {
    pub address: String,
    pub chain: String,
    pub name: Option<String>,
    pub decimals: Option<i64>,
    pub symbol: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenListData {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub decimals: i64,
}

pub struct ERC20Tokens {}

abigen!(
    ERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
    ]"#,
);

impl ERC20Tokens {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseErc20Transfer>> {
        let mut connection = db.establish_connection();

        let transfers: Result<Vec<DatabaseErc20Transfer>, Error> = erc20_transfers::table
            .select(erc20_transfers::all_columns)
            .filter(
                erc20_transfers::erc20_tokens_parsed
                    .is_null()
                    .or(erc20_transfers::erc20_tokens_parsed.eq(false)),
            )
            .limit(500)
            .load::<DatabaseErc20Transfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse_extenal(&self, db: &Database) -> Result<()> {
        let chains = get_chains();

        for (name, chain) in chains {
            info!("ERC20Tokens: fetching extenal tokens for chain {}", name);

            for url in chain.tokens_lists {
                let client = Client::new();
                let response = client.get(&**url).send().await;

                let mut tokens = vec![];

                match response {
                    Ok(data) => match data.text().await {
                        Ok(response) => {
                            let tokens_list = serde_json::from_str::<Vec<TokenListData>>(&response);
                            match tokens_list {
                                Ok(tokens_list) => {
                                    for token in tokens_list {
                                        let db_token = DatabaseErc20Token {
                                            address: token.address,
                                            chain: name.clone(),
                                            name: Some(token.name),
                                            decimals: Some(token.decimals),
                                            symbol: Some(token.symbol),
                                        };

                                        tokens.push(db_token);
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                }

                let mut connection = db.establish_connection();

                let mut query = String::from(
                    "UPSERT INTO erc20_tokens (address, chain, decimals, name, symbol) VALUES",
                );

                let tokens_amount = tokens.len();

                for token in tokens {
                    let name = match token.name {
                        Some(name) => {
                            let name_fixed: String = name.replace("'", "");

                            let name_bytes = name_fixed.as_bytes();

                            let name_parsed = String::from_utf8_lossy(name_bytes);

                            format!("'{}'", name_parsed)
                        }
                        None => String::from("NULL"),
                    };

                    let symbol = match token.symbol {
                        Some(symbol) => {
                            let symbol_fixed: String = symbol.replace("'", "");

                            let symbol_bytes = symbol_fixed.as_bytes();

                            let symbol_parsed = String::from_utf8_lossy(symbol_bytes);

                            format!("'{}'", symbol_parsed)
                        }
                        None => String::from("NULL"),
                    };

                    let value = format!(
                        " ('{}', '{}', {}, {}, {}),",
                        token.address,
                        token.chain,
                        token.decimals.unwrap(),
                        name,
                        symbol
                    );

                    query.push_str(&value);
                }

                query.pop();

                if tokens_amount > 0 {
                    sql_query(query).execute(&mut connection).unwrap();
                }

                info!(
                    "ERC20Tokens: inserted {} tokens for chain {}",
                    tokens_amount, name
                );
            }
        }
        Ok(())
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseErc20Transfer>) -> Result<()> {
        let mut connection = db.establish_connection();

        let unique_tokens: Vec<(String, String)> = transfers
            .into_iter()
            .map(|token| (token.token.clone(), token.chain.clone()))
            .collect::<HashSet<(String, String)>>()
            .into_iter()
            .collect();

        let mut tokens_data = vec![];

        for (address, chain) in unique_tokens {
            tokens_data.push(self.get_token_metadata((address, chain)))
        }

        let db_tokens: Vec<DatabaseErc20Token> = join_all(tokens_data)
            .await
            .into_iter()
            .filter(|token| token.is_some())
            .map(|token| token.unwrap())
            .collect();

        let mut query = String::from(
            "UPSERT INTO erc20_tokens (address, chain, decimals, name, symbol) VALUES",
        );

        let tokens_amount = db_tokens.len();

        for token in db_tokens {
            let name = match token.name {
                Some(name) => {
                    let name_fixed: String = name.replace("'", "");

                    let name_bytes = name_fixed.as_bytes();

                    let name_parsed = String::from_utf8_lossy(name_bytes);

                    format!("'{}'", name_parsed)
                }
                None => String::from("NULL"),
            };

            let symbol = match token.symbol {
                Some(symbol) => {
                    let symbol_fixed: String = symbol.replace("'", "");

                    let symbol_bytes = symbol_fixed.as_bytes();

                    let symbol_parsed = String::from_utf8_lossy(symbol_bytes);

                    format!("'{}'", symbol_parsed)
                }
                None => String::from("NULL"),
            };

            let value = format!(
                " ('{}', '{}', {}, {}, {}),",
                token.address,
                token.chain,
                token.decimals.unwrap(),
                name,
                symbol
            );

            query.push_str(&value);
        }

        query.pop();

        if tokens_amount > 0 {
            sql_query(query)
                .execute(&mut connection)
                .expect("Unable to store erc20 tokens data");
        }

        info!(
            "ERC20Tokens: Inserted {} erc20 tokens to the database.",
            tokens_amount
        );

        let transfers_chunks = get_chunks(transfers.len(), DatabaseErc20Transfer::field_count());

        for (start, end) in transfers_chunks {
            diesel::insert_into(erc20_transfers::dsl::erc20_transfers)
                .values(&transfers[start..end])
                .on_conflict((erc20_transfers::hash, erc20_transfers::log_index))
                .do_update()
                .set(erc20_transfers::erc20_tokens_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed erc20 transfers into database");
        }

        Ok(())
    }

    pub async fn get_token_metadata(
        &self,
        (address, chain): (String, String),
    ) -> Option<DatabaseErc20Token> {
        let chain_data = get_chain(chain.to_string());

        let provider = match Provider::<Http>::try_from(chain_data.public_rpc) {
            Ok(provider) => provider,
            Err(_) => return None,
        };

        let client = Arc::new(provider);

        let token = ERC20::new(address.parse::<Address>().unwrap(), Arc::clone(&client));

        let name: Option<String> = match token.name().call().await {
            Ok(name) => Some(format!("{}", name.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        let decimals: Option<i64> = match token.decimals().call().await {
            Ok(decimals) => Some(decimals as i64),
            Err(_) => return None,
        };

        let symbol: Option<String> = match token.symbol().call().await {
            Ok(symbol) => Some(format!("{}", symbol.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        return Some(DatabaseErc20Token {
            address,
            chain,
            name,
            decimals,
            symbol,
        });
    }
}
