use crate::db::{
    db::EVMDatabase,
    schema::{evm_erc20_balances, evm_erc20_transfers},
};
use anyhow::Result;
use diesel::{prelude::*, result::Error};
use field_count::FieldCount;

use super::erc20_transfers_parser::DatabaseEVMErc20Transfer;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = evm_erc20_balances)]
pub struct DatabaseEVMErc20Balance {
    pub address: String,
    pub chain: String,
    pub token: String,
    pub balance: String,
}

pub struct ERC20BalancesParser {}

impl ERC20BalancesParser {
    pub fn fetch(&self, db: &EVMDatabase) -> Result<Vec<DatabaseEVMErc20Transfer>> {
        let mut connection = db.establish_connection();

        let transfers: Result<Vec<DatabaseEVMErc20Transfer>, Error> = evm_erc20_transfers::table
            .select(evm_erc20_transfers::all_columns)
            .filter(
                evm_erc20_transfers::erc20_balances_parsed
                    .is_null()
                    .or(evm_erc20_transfers::erc20_balances_parsed.eq(false)),
            )
            .limit(500)
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

        for transfer in transfers {}

        Ok(())
    }
}
