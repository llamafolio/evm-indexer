use anyhow::Result;
use log::*;
use web3::futures::future::join_all;

use crate::{db::Database, rpc::Rpc};

pub async fn fetch_blocks(
    rpc: &Rpc,
    db: &Database,
    from: i64,
    to: i64,
    batch_size: usize,
    workers: usize,
) -> Result<()> {
    info!(
        "Fetching block range from {} to {} with batches of {} blocks with {} workers",
        from, to, batch_size, workers
    );

    let range: Vec<i64> = (from..to).collect();

    for work_chunk in range.chunks(batch_size * workers) {
        let mut works = vec![];

        let chunks = work_chunk.chunks(batch_size.clone());

        info!(
            "Procesing chunk from block {} to {}",
            work_chunk.first().unwrap(),
            work_chunk.last().unwrap()
        );

        for worker_part in chunks {
            works.push(rpc.get_block_batch(worker_part.to_vec()));
        }

        let res = join_all(works)
            .await
            .into_iter()
            .map(Result::unwrap)
            .flatten()
            .collect();

        db.store_blocks(res, true).await;
    }

    Ok(())
}
