use std::{collections::HashSet, sync::Arc};

use crate::{
    chains::chains::get_chain,
    db::{
        db::{get_chunks, EVMDatabase},
        schema::{evm_erc20_tokens, evm_erc20_transfers, evm_transactions},
    },
};
use anyhow::Result;
use diesel::{prelude::*, result::Error};
use ethabi::Address;
use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
};
use field_count::FieldCount;
use futures::future::join_all;
use log::info;

use super::erc20_transfers_parser::DatabaseEVMErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = evm_erc20_tokens)]
pub struct DatabaseEVMErc20Token {
    pub address: String,
    pub chain: String,
    pub name: Option<String>,
    pub decimals: Option<i64>,
    pub symbol: Option<String>,
}

pub struct ERC20TokensParser {}

abigen!(
    ERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
    ]"#,
);

impl ERC20TokensParser {
    pub fn fetch(&self, db: &EVMDatabase) -> Result<Vec<DatabaseEVMErc20Transfer>> {
        let mut connection = db.establish_connection();

        let transfers: Result<Vec<DatabaseEVMErc20Transfer>, Error> = evm_erc20_transfers::table
            .select(evm_erc20_transfers::all_columns)
            .filter(
                evm_erc20_transfers::erc20_tokens_parsed
                    .is_null()
                    .or(evm_erc20_transfers::erc20_tokens_parsed.eq(false)),
            )
            .limit(5000)
            .load::<DatabaseEVMErc20Transfer>(&mut connection);

        match transfers {
            Ok(transfers) => Ok(transfers),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn parse(
        &self,
        db: &EVMDatabase,
        transfers: &Vec<DatabaseEVMErc20Transfer>,
    ) -> Result<()> {
        let mut connection = db.establish_connection();

        let unique_tokens: Vec<String> = transfers
            .into_iter()
            .map(|transfer| {
                let chain: String = evm_transactions::table
                    .select(evm_transactions::chain)
                    .filter(evm_transactions::hash.eq(transfer.hash.clone()))
                    .first::<String>(&mut connection)
                    .unwrap();

                return format!("{}-{}", transfer.token, chain);
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let mut tokens_data = vec![];

        for token in unique_tokens {
            tokens_data.push(self.get_token_metadata(token))
        }

        let db_tokens: Vec<DatabaseEVMErc20Token> = join_all(tokens_data)
            .await
            .into_iter()
            .filter(|token| token.is_some())
            .map(|token| token.unwrap())
            .collect();

        let chunks = get_chunks(db_tokens.len(), DatabaseEVMErc20Token::field_count());

        for (start, end) in chunks {
            diesel::insert_into(evm_erc20_tokens::dsl::evm_erc20_tokens)
                .values(&db_tokens[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store erc20 tokens into database");
        }

        info!("Inserted {} erc20 tokens to the database.", db_tokens.len());

        let transfers_chunks = get_chunks(transfers.len(), DatabaseEVMErc20Transfer::field_count());

        for (start, end) in transfers_chunks {
            diesel::insert_into(evm_erc20_transfers::dsl::evm_erc20_transfers)
                .values(&transfers[start..end])
                .on_conflict((evm_erc20_transfers::hash, evm_erc20_transfers::log_index))
                .do_update()
                .set(evm_erc20_transfers::erc20_tokens_parsed.eq(true))
                .execute(&mut connection)
                .expect("Unable to update parsed erc20 transfers into database");
        }

        Ok(())
    }

    async fn get_token_metadata(&self, token_id: String) -> Option<DatabaseEVMErc20Token> {
        let address_chain: Vec<&str> = token_id.split("-").collect();

        let address = address_chain[0];
        let chain = address_chain[1];

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
            Ok(decimals) => Some(decimals.into()),
            Err(_) => None,
        };

        let symbol: Option<String> = match token.symbol().call().await {
            Ok(symbol) => Some(format!("{}", symbol.trim_matches(char::from(0)))),
            Err(_) => None,
        };

        return Some(DatabaseEVMErc20Token {
            address: address_chain[0].to_string(),
            chain: address_chain[1].to_string(),
            name,
            decimals,
            symbol,
        });
    }
}
