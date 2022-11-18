use anyhow::Result;
use tokio_postgres::{ Client, NoTls };
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use web3::types::{ Transaction, Block};

#[derive(PostgresMapper)]
#[pg_mapper(table = "state")]
pub struct State {
    pub id: String,
    pub last_block: i64,
}

const CREATE_STATE_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS sync_state (
    id VARCHAR NOT NULL UNIQUE,
    last_block BIGINT
  ); 
";

#[derive(PostgresMapper)]
#[pg_mapper(table = "blocks")]
pub struct DatabaseBlock {
    pub height: i64,
    pub txs: i64,
    pub timestamp: i64
}

const CREATE_BLOCKS_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS blocks (
        height BIGINT UNIQUE,
        txs BIGINT,
        timestamp BIGINT
  ); 
";

pub struct IndexerDB {
    pub db: Client,
}

impl IndexerDB {
    pub async fn new(db_url: &str) -> Result<Self> {
        log::info!("==> IndexerDB: Initializing IndexerDB");

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

        client
            .query(CREATE_BLOCKS_TABLE, &[]).await
            .expect("Unable to run blocks creation query");

        Ok(IndexerDB {
            db: client,
        })
    }

    pub async fn last_synced_block(&self) -> Result<i64> {
        let query = &self.db.query("SELECT * from sync_state", &[]).await.unwrap();

        // Get the first row to fetch the data
        let row = query.get(0);

        match row {
            None => {
                // If no data, initialize the table
                let _ = &self.db
                    .query(
                        "INSERT INTO sync_state(id, last_block) VALUES ('sync_state', 100000)",
                        &[]
                    ).await
                    .expect("Unable to write initial state data");

                Ok(100000)
            }
            Some(row) => {
                let state = State::from_row_ref(row).unwrap();
                Ok(state.last_block)
            }
        }
    }

    pub async fn store_block_batch(&self, blocks: Vec<Block<Transaction>>)  {
        let mut query: String = String::from("INSERT INTO blocks(height,txs, timestamp) VALUES ");

        for (i, block) in blocks.iter().enumerate() {
            let values = format!("({},{},{})", block.number.unwrap().as_u64() as i64, block.transactions.len() as i64,block.timestamp.as_u64() as i64);
            if i > 0 {
                query.push_str(&",");
            }
            query.push_str(&values);
        }

        let _ = &self.db.query(&query, &[]).await.expect("Unable to store block batch");

        log::info!("==> IndexerDB: Stored {} blocks", blocks.len());

        let last_block = blocks.last().unwrap().number.unwrap().as_u64() as i64;


        self.update_sync_state(last_block).await;
    }

    pub async fn update_sync_state(&self, last_block: i64)  {
        let query = format!("UPDATE sync_state SET last_block = {} WHERE id = 'sync_state' ", last_block);

        let _ = &self.db.query(&query, &[]).await.expect("Unable to update last block sync state");

        log::info!("==> IndexerDB: Updated sync state to block {}", last_block);

    }

    pub async fn store_block(&self, block: Block<Transaction>) {
        
        let block: DatabaseBlock = DatabaseBlock {
            height: block.number.unwrap().as_u64() as i64,
            txs: block.transactions.len() as i64,
            timestamp: block.timestamp.as_u64() as i64
        };

        let _ = &self.db.query("INSERT INTO blocks(height, txs, timestamp) VALUES ($1, $2, $3)", &[&block.height, &block.txs, &block.timestamp]).await;
    }


}