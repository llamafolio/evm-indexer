use anyhow::Result;
use log::*;
use web3::{
    futures::future::join_all,
    types::{Block, Transaction},
    Error,
};

use crate::{
    db::models::{DatabaseBlock, DatabaseTx, DatabaseTxLogs, DatabaseTxReceipt},
    db::Database,
    rpc::Rpc,
    utils::format_block,
};

pub async fn store_blocks(db: &Database, blocks: Vec<Block<Transaction>>) {
    db.store_blocks(&blocks).await;

    let txs: Vec<DatabaseTx> = blocks
        .into_iter()
        .map(|block| {
            block
                .transactions
                .into_iter()
                .map(|tx| DatabaseTx::from_web3(tx))
        })
        .flatten()
        .collect();

    db.store_txs(&txs).await;
}

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

        let web3_blocks: Vec<Block<Transaction>> = join_all(works)
            .await
            .into_iter()
            .map(Result::unwrap)
            .flatten()
            .collect();

        let web3_txs: Vec<Transaction> = web3_blocks
            .into_iter()
            .map(|block| block.transactions)
            .flatten()
            .collect();

        let web3_receipts = rpc.get_txs_receipts(web3_txs).await.into_iter().flatten();

        let db_blocks: Vec<DatabaseBlock> = web3_blocks
            .into_iter()
            .map(|block| DatabaseBlock::from_web3(&block))
            .collect();

        let db_txs: Vec<DatabaseTx> = web3_txs
            .into_iter()
            .map(|tx| DatabaseTx::from_web3(&tx))
            .collect();

        let db_tx_receipts: Vec<DatabaseTxReceipt> = web3_receipts
            .into_iter()
            .map(|tx_receipt| DatabaseTxReceipt::from_web3(&tx_receipt))
            .collect();

        let db_tx_logs: Vec<DatabaseTxLogs> = web3_receipts
            .into_iter()
            .map(|tx_receipt| tx_receipt.logs)
            .flatten()
            .map(|tx_log| DatabaseTxLogs::from_web3(tx_log))
            .collect();

        db.store_blocks_and_txs(db_blocks, db_txs, db_tx_receipts, db_tx_logs)
            .await;
    }

    Ok(())
}
