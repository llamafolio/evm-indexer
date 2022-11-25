use std::collections::HashSet;

use anyhow::Result;
use log::*;
use web3::futures::future::join_all;

use crate::{config::Config, db::Database, rpc::Rpc};

pub async fn fetch_blocks(rpc: &Rpc, db: &Database, config: Config) -> Result<()> {
    let rpc_last_block = rpc.get_last_block().await.unwrap();

    let full_blocks_set: Vec<i64> = (config.start_block..rpc_last_block).collect();

    let db_blocks_set = vec_to_set(db.get_block_numbers().await.unwrap());

    let missing_blocks_set: Vec<i64> = full_blocks_set
        .into_iter()
        .filter(|n| !db_blocks_set.contains(n))
        .collect();

    info!(
        "Fetching {} blocks with batches of {} blocks with {} workers",
        missing_blocks_set.len(),
        config.batch_size,
        config.workers
    );

    for work_chunk in missing_blocks_set.chunks(config.batch_size * config.workers) {
        let mut works = vec![];

        let chunks = work_chunk.chunks(config.batch_size);

        info!(
            "Procesing chunk from block {} to {}",
            work_chunk.first().unwrap(),
            work_chunk.last().unwrap()
        );

        for worker_part in chunks {
            works.push(rpc.get_blocks(worker_part.to_vec()));
        }

        let block_responses = join_all(works).await;

        let res = block_responses.into_iter().map(Result::unwrap);

        if res.len() < config.workers {
            info!("Incomplete result returned, omitting...")
        }

        let mut stores = vec![];

        for (
            db_blocks,
            db_txs,
            db_tx_receipts,
            db_tx_logs,
            db_contract_creation,
            db_contract_interaction,
            db_token_transfers,
        ) in res
        {
            if db_blocks.len() < config.batch_size {
                info!("Incomplete blocks returned, omitting...");
                continue;
            }

            if db_txs.len() != db_tx_receipts.len() {
                info!("Txs and txs_receipts don't match, omitting...");
                continue;
            }

            stores.push(db.store_blocks_and_txs(
                db_blocks,
                db_txs,
                db_tx_receipts,
                db_tx_logs,
                db_contract_creation,
                db_contract_interaction,
                db_token_transfers,
            ));
        }

        join_all(stores).await;
    }

    Ok(())
}

fn vec_to_set(vec: Vec<i64>) -> HashSet<i64> {
    HashSet::from_iter(vec)
}
