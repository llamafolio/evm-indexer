use std::str::FromStr;

use crate::db::{
    db::{get_chunks, Database},
    schema::{nft_token_uris, logs},
};
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};
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
            .limit(500)
            .load::<DatabaseNftTokenUri>(&mut connection);

        match tokens {
            Ok(tokens) => Ok(tokens),
            Err(_) => Ok(Vec::new()),
        }
    }
}
