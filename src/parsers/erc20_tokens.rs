use std::{collections::HashSet, sync::Arc};

use crate::{
    chains::chains::{get_chain, get_chains},
    db::db::{get_chunks, Database},
};
use anyhow::Result;
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
use sqlx::QueryBuilder;

use super::erc20_transfers::DatabaseErc20Transfer;

#[derive(Debug, Clone, FieldCount, sqlx::FromRow)]
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
    pub async fn fetch(&self, db: &Database) -> Result<Vec<DatabaseErc20Transfer>> {
        let connection = db.get_connection();

        let rows = sqlx::query_as::<_, DatabaseErc20Transfer>(
            "SELECT * FROM erc20_transfers transfer WHERE NOT EXISTS (SELECT 1 FROM erc20_tokens token WHERE transfer.chain = token.chain AND transfer.token = token.address) LIMIT 500",
        )
        .fetch_all(connection)
        .await;

        match rows {
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

                let connection = db.get_connection();

                let mut query_builder = QueryBuilder::new(
                    "UPSERT INTO erc20_tokens (address, chain, decimals, name, symbol) ",
                );

                let tokens_amount = tokens.len();

                let mut tokens_data = vec![];

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

                    tokens_data.push((
                        token.address,
                        token.chain,
                        token.decimals.unwrap(),
                        name,
                        symbol,
                    ));
                }

                query_builder.push_values(
                    &tokens_data,
                    |mut row, (address, chain, decimals, name, symbol)| {
                        row.push_bind(address)
                            .push_bind(chain)
                            .push_bind(decimals)
                            .push_bind(name)
                            .push_bind(symbol);
                    },
                );

                if tokens_amount > 0 {
                    let query = query_builder.build();

                    query
                        .execute(connection)
                        .await
                        .expect("Unable to store erc20 balances into database");
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
        let connection = db.get_connection();

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

        let mut query_builder =
            QueryBuilder::new("UPSERT INTO erc20_tokens (address, chain, decimals, name, symbol) ");

        let tokens_amount = db_tokens.len();

        let mut tokens_data = vec![];

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

            tokens_data.push((
                token.address,
                token.chain,
                token.decimals.unwrap(),
                name,
                symbol,
            ));
        }

        query_builder.push_values(
            &tokens_data,
            |mut row, (address, chain, decimals, name, symbol)| {
                row.push_bind(address)
                    .push_bind(chain)
                    .push_bind(decimals)
                    .push_bind(name)
                    .push_bind(symbol);
            },
        );

        if tokens_amount > 0 {
            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to store erc20 balances into database");
        }

        info!(
            "ERC20Tokens: Inserted {} erc20 tokens to the database.",
            tokens_amount
        );

        let transfers_chunks = get_chunks(transfers.len(), DatabaseErc20Transfer::field_count());

        for (start, end) in transfers_chunks {
            let mut query_builder =
                QueryBuilder::new("UPSERT INTO erc20_transfers(chain, from_address, hash, log_index, to_address, token, value) ");

            query_builder.push_values(&transfers[start..end], |mut row, erc20_transfer| {
                row.push_bind(erc20_transfer.chain.clone())
                    .push_bind(erc20_transfer.from_address.clone())
                    .push_bind(erc20_transfer.hash.clone())
                    .push_bind(erc20_transfer.log_index.clone())
                    .push_bind(erc20_transfer.to_address.clone())
                    .push_bind(erc20_transfer.token.clone())
                    .push_bind(erc20_transfer.value.clone());
            });

            let query = query_builder.build();

            query
                .execute(connection)
                .await
                .expect("Unable to update erc20 transfers into database");
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
            Err(_) => Some(0),
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
