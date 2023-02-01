use std::sync::Arc;

use crate::{
    chains::chains::{get_chain, get_chains},
    db::{
        db::{get_chunks, Database},
        schema::{nft_tokens, nft_transfers},
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
use itertools::Itertools;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::nft_transfers::DatabaseNftTransfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount, QueryableByName)]
#[diesel(table_name = nft_tokens)]
pub struct DatabaseNftToken {
    pub address: String,
    pub chain: String,
    pub nft_type: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub contract_uri: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenListData {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub decimals: i64,
}

pub struct NftTokens {}

// We can also use this with ERC1155
abigen!(
    ERC721,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function contractURI() external view returns (string)
        function tokenURI(uint256 tokenId) external view returns (string)
    ]"#,
);

impl NftTokens {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseNftTransfer>> {
        let mut connection = db.establish_connection();

        let transfers: Result<Vec<DatabaseNftTransfer>, Error> = nft_transfers::table
            .select(nft_transfers::all_columns)
            .filter(
                nft_transfers::nft_tokens_parsed
                    .is_null()
                    .or(nft_transfers::nft_tokens_parsed.eq(false)),
            )
            .limit(500)
            .load::<DatabaseNftTransfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, transfers: &Vec<DatabaseNftTransfer>) -> Result<()> {
        let mut connection = db.establish_connection();

        let unique_tokens: Vec<String> = transfers
            .into_iter()
            .map(|token| format!("{}-{}-{}", token.token, token.chain, token.transfer_type))
            .unique()
            .collect();

        let mut tokens_data = vec![];

        for token in unique_tokens {
            tokens_data.push(self.get_token_metadata(token))
        }

        let db_tokens: Vec<DatabaseNftToken> = join_all(tokens_data)
            .await
            .into_iter()
            .filter(|token| token.is_some())
            .map(|token| token.unwrap())
            .collect();

        let mut query = String::from(
            "UPSERT INTO nft_tokens (address, chain, nft_type, name, symbol, contract_uri) VALUES",
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
                None => String::from(""),
            };

            let symbol = match token.symbol {
                Some(symbol) => {
                    let symbol_fixed: String = symbol.replace("'", "");

                    let symbol_bytes = symbol_fixed.as_bytes();

                    let symbol_parsed = String::from_utf8_lossy(symbol_bytes);

                    format!("'{}'", symbol_parsed)
                }
                None => String::from(""),
            };

            // TODO: Must find a bettwe way to encode SQL. It's too danger to do this way
            let contract_uri = match token.contract_uri {
                Some(contract_uri) => {
                    let contract_uri_fixed: String = contract_uri.replace("'", "");

                    let contract_uri_bytes = contract_uri_fixed.as_bytes();

                    let contract_uri_parsed = String::from_utf8_lossy(contract_uri_bytes);

                    format!("'{}'", contract_uri_parsed)
                }
                None => String::from(""),
            };

            let value = format!(
                " ('{}', '{}', '{}', {}, {}, {}),",
                token.address,
                token.chain,
                token.nft_type,
                name,
                symbol,
                contract_uri,
            );

            query.push_str(&value);
        }

        query.pop();

        if tokens_amount > 0 {
            sql_query(query).execute(&mut connection).unwrap();
        }

        info!("Inserted {} nft tokens to the database.", tokens_amount);

        let transfers_chunks = get_chunks(transfers.len(), DatabaseNftTransfer::field_count());

        for (start, end) in transfers_chunks {
            diesel::insert_into(nft_transfers::dsl::nft_transfers)
                .values(&transfers[start..end])
                .on_conflict((nft_transfers::hash, nft_transfers::log_index))
                .do_update()
                .set(nft_transfers::nft_tokens_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed nft transfers into database");
        }

        Ok(())
    }

    pub async fn get_token_metadata(&self, token_id: String) -> Option<DatabaseNftToken> {
        let address_chain: Vec<&str> = token_id.split("-").collect();

        let address = address_chain[0];
        let chain = address_chain[1];
        let transfer_type = address_chain[2];

        let chain_data = get_chain(chain.to_string());

        let provider = match Provider::<Http>::try_from(chain_data.public_rpc) {
            Ok(provider) => provider,
            Err(_) => return None,
        };

        let client = Arc::new(provider);

        let token = ERC721::new(address.parse::<Address>().unwrap(), Arc::clone(&client));

        let name: Option<String> = match token.name().call().await {
            Ok(name) => Some(format!("{}", name.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        let symbol: Option<String> = match token.symbol().call().await {
            Ok(symbol) => Some(format!("{}", symbol.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        let contract_uri: Option<String> = match token.contract_uri().call().await {
            Ok(contract_uri) => Some(format!("{}", contract_uri.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        let nft_type = match transfer_type {
            "ERC721Transfer" => "ERC721",
            "ERC1155TransferSingle" => "ERC1155",
            "ERC1155TransferBatch" => "ERC1155",
            &_ => "",
        };

        return Some(DatabaseNftToken {
            address: address_chain[0].to_string(),
            chain: address_chain[1].to_string(),
            nft_type: nft_type.to_owned(),
            name,
            symbol,
            contract_uri,
        });
    }
}
