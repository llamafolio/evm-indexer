use web3::futures::future::join_all;

use crate::db::IndexerDB;
use crate::rpc::IndexerRPC;

async fn fetch_blocks_range(
    rpc: &IndexerRPC,
    db: &IndexerDB,
    chunk: &[i64],
    update_sync_state: bool,
) {
    log::info!(
        "==> Main: Procesing chunk from block {} to {}",
        chunk.first().unwrap(),
        chunk.last().unwrap()
    );

    let blocks = rpc.fetch_block_batch(chunk).await.unwrap();

    if blocks.len() > 0 {
        db.store_block_batch(blocks, update_sync_state).await;
    }
}

pub async fn fetch_blocks_range_workers(
    rpc: &IndexerRPC,
    db: &IndexerDB,
    from: i64,
    to: i64,
    batch_size: &usize,
    workers: &usize,
) {
    log::info!(
        "==> Main: Fetching block range from {} to {} with batches of {} blocks with {} workers",
        from,
        to,
        batch_size,
        workers
    );

    let full_block_range: Vec<i64> = (from..to).collect();

    for work_chunk in full_block_range.chunks(batch_size * workers) {
        let mut works = vec![];

        let chunks = work_chunk.chunks(batch_size.clone());

        let chunks_size = chunks.len();

        for (i, worker_part) in chunks.enumerate() {
            works.push(fetch_blocks_range(
                rpc,
                db,
                worker_part,
                i == chunks_size - 1,
            ));
        }

        join_all(works).await;
    }
}
