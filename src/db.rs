use anyhow::{ Result };
use tokio_postgres::{ Client, NoTls };

struct State {
    id: String,
    last_block: i64,
}

const CREATE_STATE_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS sync_state (
    id VARCHAR NOT NULL UNIQUE,
    last_block BIGINT
  ); 
";

struct Block {
    id: i64,
    hash: String,
    txs: i64,
    timestamp: i64,
}

pub struct IndexerDB {
    pub db: Client,
}

impl IndexerDB {
    pub async fn new(db_url: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // Create tables if required
        client
            .query(CREATE_STATE_TABLE, &[]).await
            .expect("Unable to run sync_state creation query");

        Ok(IndexerDB {
            db: client,
        })
    }

    pub async fn last_synced_block(&self) -> Result<i64> {
        let query = &self.db.query("SELECT last_block from sync_state", &[]).await.unwrap();

        // Get the first row to fetch the data
        let row = query.get(0);

        match row {
            None => {
                // If no data, initialize the table
                let _ = &self.db
                    .query(
                        "INSERT INTO sync_state(id, last_block) VALUES ('sync_state', 0)",
                        &[]
                    ).await
                    .expect("Unable to write initial state data");
                Ok(0)
            }
            Some(row) => {
                let last_block = row.try_get::<usize, i64>(0).expect("Unable to fetch last_block from row");
                Ok(last_block)
            }
        }
    }
}