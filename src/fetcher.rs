use std::collections::HashSet;

use anyhow::Result;
use log::*;

use crate::{config::Config, db::Database, rpc::Rpc};

pub async fn fetch_blocks(rpc: &Rpc, db: &Database, config: Config) -> Result<()> {
    let rpc_last_block = rpc.get_last_block().await.unwrap();

    let full_blocks_set: Vec<i64> = (config.start_block..rpc_last_block).collect();

    let db_blocks_set = vec_to_set(db.get_block_numbers().await.unwrap());

    let missing_blocks: Vec<i64> = full_blocks_set
        .into_iter()
        .filter(|n| !db_blocks_set.contains(n))
        .collect();

    let missing_blocks_amount = missing_blocks.len();

    info!(
        "Fetching {} blocks with batches of {} blocks with {} workers",
        missing_blocks_amount, config.batch_size, config.workers
    );

    let chunks = missing_blocks.chunks(config.batch_size);

    for chunk in chunks {
        let chunk_vec = chunk.to_vec();

        info!(
            "Procesing chunk from block {} to {} for chain {}",
            chunk_vec.first().unwrap(),
            chunk_vec.last().unwrap(),
            config.chain
        );

        let (
            db_blocks,
            db_txs,
            db_tx_receipts,
            db_tx_logs,
            db_contract_creation,
            db_contract_interaction,
            db_token_transfers,
        ) = rpc.get_blocks(chunk_vec.to_vec()).await.unwrap();

        if db_blocks.len() < chunk_vec.len() {
            info!("Incomplete blocks returned, omitting...");
            continue;
        }

        if db_txs.len() != db_tx_receipts.len() {
            info!("Txs and txs_receipts don't match, omitting...");
            continue;
        }

        db.store_blocks_and_txs(
            db_blocks,
            db_txs,
            db_tx_receipts,
            db_tx_logs,
            db_contract_creation,
            db_contract_interaction,
            db_token_transfers,
        )
        .await;
    }

    Ok(())
}

fn vec_to_set(vec: Vec<i64>) -> HashSet<i64> {
    HashSet::from_iter(vec)
}
