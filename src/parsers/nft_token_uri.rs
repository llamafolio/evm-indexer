use std::str::FromStr;

use crate::{db::{
    db::{get_chunks, Database},
    schema::{nft_token_uris, logs},
}, chains::chains::get_chain};
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::{prelude::*, result::Error, sql_query, upsert::excluded};
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
use std::sync::Arc;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = nft_token_uris)]
pub struct DatabaseNftTokenUri {
    pub token: String,
    pub token_id: BigDecimal,
    pub chain: String,
    pub token_uri: Option<String>,
    pub is_parsed: bool,
}

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

pub struct NftTokenUri {}

impl NftTokenUri {
    pub fn fetch(&self, db: &Database) -> Result<Vec<DatabaseNftTokenUri>> {
        let mut connection = db.establish_connection();

        let tokens: Result<Vec<DatabaseNftTokenUri>, Error> = nft_token_uris::table
            .select(nft_token_uris::all_columns)
            .filter(
                nft_token_uris::is_parsed
                    .is_null()
                    .or(nft_token_uris::is_parsed.eq(false)),
            )
            .limit(100)
            .load::<DatabaseNftTokenUri>(&mut connection);

        match tokens {
            Ok(tokens) => Ok(tokens),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(&self, db: &Database, token_uris: &mut Vec<DatabaseNftTokenUri>) -> Result<()> {
        let mut connection = db.establish_connection();
        
        let mut token_uris_future = vec![];
        
        for token in token_uris.clone() {
            token_uris_future.push(self.get_token_uri(token.token.clone(), token.chain.clone(), token.token_id.clone()));
        }

        let token_uris_result = join_all(token_uris_future).await;

        for i in 1..token_uris.len() {
            token_uris[i].token_uri = token_uris_result[i].clone();
            token_uris[i].is_parsed = true;
        }

        let transfers_chunks = get_chunks(token_uris.len(), DatabaseNftTokenUri::field_count());

        for (start, end) in transfers_chunks {
            diesel::insert_into(nft_token_uris::dsl::nft_token_uris)
                .values(&token_uris[start..end])
                .on_conflict((nft_token_uris::token, nft_token_uris::token_id, nft_token_uris::chain))
                .do_update()
                .set((
                    nft_token_uris::token_uri.eq(excluded(nft_token_uris::token_uri)),
                    nft_token_uris::is_parsed.eq(true),
                ))
                .execute(&mut connection)
                .expect("Unable to update parsed nft token uri into database");
        }

        Ok(())
    }

    pub async fn get_token_uri(&self, address: String, chain: String, token_id: BigDecimal) -> Option<String> {
        let token_id_str = token_id.to_string();

        let chain_data = get_chain(chain.to_string());

        // TODO: Should use private rpc because of rate limiting
        let provider = match Provider::<Http>::try_from(chain_data.public_rpc) {
            Ok(provider) => provider,
            Err(_) => return None,
        };

        let client = Arc::new(provider);

        let token = ERC721::new(address.parse::<Address>().unwrap(), Arc::clone(&client));

        let token_id_ethers = match ethers::types::U256::from_str(&token_id_str) {
            Ok(token_id) => token_id,
            Err(_) => return None,
        };

        let token_uri: Option<String> = match token.token_uri(token_id_ethers).call().await {
            Ok(token_uri) => Some(format!("{}", token_uri.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        return token_uri;
    }
}
