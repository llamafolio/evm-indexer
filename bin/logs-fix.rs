use diesel::{connection::SimpleConnection, prelude::*};
use dotenv::dotenv;
use evm_indexer::{
    chains::chains::ETHEREUM,
    db::{
        db::Database,
        models::models::DatabaseLog,
        schema::{logs, transactions},
    },
};
use futures::future::join_all;

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let db = Database::new(
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
        std::env::var("REDIS_URL").expect("REDIS_URL must be set."),
        ETHEREUM,
    )
    .await
    .unwrap();

    println!("Fixing logs chain info");

    loop {
        let mut connection = db.establish_connection();

        let txs: Vec<(String, String)> = transactions::dsl::transactions
            .select((transactions::hash, transactions::chain))
            .filter(transactions::parsed.eq(false))
            .limit(1000)
            .load::<(String, String)>(&mut connection)
            .unwrap();

        if txs.len() <= 0 {
            panic!("Finished");
        }

        println!("Fetched {} transactions to fix", txs.len());

        let mut works = vec![];

        let mut query = String::from("");

        for (hash, chain) in txs {
            works.push(process_tx(&db, hash.clone(), chain));
            let sql = format!(
                "UPDATE transactions SET parsed = true WHERE hash = '{}';",
                hash
            );
            query.push_str(&sql);
        }

        join_all(works).await;
        if query != String::from("") {
            connection.batch_execute(&query).unwrap();
        }
    }
}

async fn process_tx(db: &Database, tx: String, chain: String) {
    let mut connection = db.establish_connection();

    let logs: Vec<DatabaseLog> = logs::dsl::logs
        .select(logs::all_columns)
        .filter(logs::hash.eq(tx))
        .load::<DatabaseLog>(&mut connection)
        .unwrap();

    diesel::insert_into(logs::table)
        .values(&logs)
        .on_conflict((logs::hash, logs::log_index))
        .do_update()
        .set(logs::chain.eq(chain))
        .execute(&mut connection)
        .expect("Unable to update logs");

    println!("Processed {} logs", logs.len());
}
