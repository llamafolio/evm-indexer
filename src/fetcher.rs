use std::{collections::HashSet, time::Duration};

use anyhow::Result;
use log::*;
use tokio::time::sleep;
use web3::futures::future::join_all;

use crate::{
    config::Config,
    db::{
        models::{DatabaseExcludedToken, DatabaseToken, DatabaseTx, DatabaseTxNoReceipt},
        Database,
    },
    rpc::Rpc,
    utils::format_hash,
};

pub async fn fetch_blocks(providers: &Vec<Rpc>, db: &Database, config: &Config) -> Result<()> {
    let rpc_last_block = providers[0].get_last_block().await.unwrap();

    let full_blocks_set: Vec<i64> = (config.start_block..rpc_last_block).collect();

    let db_blocks_set = vec_to_set(db.get_block_numbers().await.unwrap());

    let missing_blocks: Vec<i64> = full_blocks_set
        .into_iter()
        .filter(|n| !db_blocks_set.contains(n))
        .collect();

    let missing_blocks_amount = missing_blocks.len();
    let providers_amount = providers.len();

    info!(
        "Fetching {} blocks with batches of {} blocks with {} workers from {} providers",
        missing_blocks_amount, config.batch_size, config.workers, providers_amount
    );

    let providers_chunk: Vec<Vec<i64>> = missing_blocks
        .clone()
        .chunks(missing_blocks_amount / providers_amount)
        .into_iter()
        .map(|chunk| chunk.to_vec())
        .collect();

    let mut providers_work = vec![];
    for (i, provider) in providers.into_iter().enumerate() {
        let provider_work = tokio::spawn({
            let chunk = providers_chunk[i].clone();
            let rpc = provider.clone();
            let db = db.clone();
            let config = config.clone();

            async move {
                for work_chunk in chunk.chunks(config.batch_size * config.workers) {
                    let mut works = vec![];

                    let chunks = work_chunk.chunks(config.batch_size);

                    info!(
                        "Procesing chunk from block {} to {} for chain {}",
                        work_chunk.first().unwrap(),
                        work_chunk.last().unwrap(),
                        config.chain.name
                    );

                    for worker_part in chunks {
                        works.push(rpc.get_blocks(worker_part.to_vec()));
                    }

                    let block_responses = join_all(works).await;

                    let res = block_responses.into_iter().map(Result::unwrap);

                    if res.len() < config.workers {
                        info!("Incomplete result returned, omitting...")
                    }

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
                        let db_txs_count = db_txs.len();
                        let db_tx_receipts_count = db_tx_receipts.len();

                        if db_txs_count != db_tx_receipts_count {
                            info!(
                                "Not enough receipts for batch: txs({}) receipts ({}) block_range({})-({})",
                                db_txs_count,
                                db_tx_receipts_count,
                                db_blocks.first().unwrap().number,
                                db_blocks.last().unwrap().number,
                            );
                        }

                        let db_receipts_hash: Vec<String> = db_tx_receipts
                            .clone()
                            .into_iter()
                            .map(|receipt| receipt.hash)
                            .collect();

                        let mut db_txs_with_receipts: Vec<DatabaseTx> = vec![];
                        let mut db_txs_with_no_receipts: Vec<DatabaseTxNoReceipt> = vec![];

                        for tx in db_txs {
                            if db_receipts_hash.contains(&tx.hash) {
                                db_txs_with_receipts.push(tx);
                            } else {
                                db_txs_with_no_receipts.push(DatabaseTxNoReceipt {
                                    hash: tx.hash,
                                    chain: tx.chain,
                                    block_number: tx.block_number,
                                });
                            }
                        }

                        db.store_blocks_and_txs(
                            db_blocks,
                            db_txs_with_receipts,
                            db_tx_receipts,
                            db_tx_logs,
                            db_contract_creation,
                            db_contract_interaction,
                            db_token_transfers,
                        )
                        .await;

                        if db_txs_with_no_receipts.len() > 0 {
                            info!(
                                "Storing {} txs with no receipt for future check",
                                db_txs_with_no_receipts.len(),
                            );

                            db.store_txs_no_receipt(&db_txs_with_no_receipts).await;
                        }
                    }
                }
            }
        });
        providers_work.push(provider_work);
    }

    join_all(providers_work).await;

    Ok(())
}

pub async fn fetch_tokens_metadata(rpc: &Rpc, db: &Database, config: &Config) -> Result<()> {
    let missing_tokens = db.get_tokens_missing_data().await.unwrap();

    let chunks = missing_tokens.chunks(100);

    for chunk in chunks {
        let data = rpc.get_tokens_metadata(chunk.to_vec()).await.unwrap();

        let added_tokens = data.len();

        let filtered_tokens: Vec<DatabaseToken> = data
            .clone()
            .into_iter()
            .filter(|token| token.name != String::from("") && token.symbol != String::from(""))
            .filter(|token| {
                !token.name.as_bytes().contains(&0x00) && !token.symbol.as_bytes().contains(&0x00)
            })
            .collect();

        db.store_tokens(&filtered_tokens).await.unwrap();

        info!("Stored data for {} tokens", added_tokens);

        let included_addresses: Vec<String> = data.into_iter().map(|token| token.address).collect();

        let excluded = chunk
            .into_iter()
            .filter(|token| !included_addresses.contains(token))
            .map(|excluded| DatabaseExcludedToken {
                address: excluded.to_string(),
                address_with_chain: format!("{}-{}", excluded.to_string(), config.chain.name),
                chain: config.chain.name.to_string(),
            })
            .collect();

        db.store_excluded_tokens(&excluded).await.unwrap();

        info!("Stored data for {} excluded tokens", excluded.len());
    }

    Ok(())
}

pub async fn fetch_tx_no_receipts(rpc: &Rpc, config: &Config, db: &Database) -> Result<()> {
    let missing_txs = db.get_missing_receipts_txs().await.unwrap();

    info!(
        "Fetching {} transactions with no receipts in shorter batches for chain {}",
        missing_txs.len(),
        config.chain.name
    );

    if missing_txs.len() == 0 {
        sleep(Duration::from_secs(120)).await;
        return Ok(());
    }

    let chunks: Vec<Vec<String>> = missing_txs
        .chunks(50)
        .into_iter()
        .map(|chunk| chunk.to_vec())
        .collect();

    let mut works = vec![];

    for n in 0..5 {
        let work = tokio::spawn({
            let work_chunk = chunks[n as usize].clone();

            let db = db.clone();
            let rpc = rpc.clone();

            async move {
                let tx_receipts = rpc.get_txs_receipts(&work_chunk.to_vec()).await.unwrap();

                if tx_receipts.len() == 0 {
                    return;
                }

                let (
                    db_tx_receipts,
                    db_tx_logs,
                    db_contract_creations,
                    db_contract_interactions,
                    db_token_transfers,
                ) = rpc
                    .get_metadata_from_receipts(tx_receipts.clone())
                    .await
                    .unwrap();

                db.store_blocks_and_txs(
                    Vec::new(),
                    Vec::new(),
                    db_tx_receipts,
                    db_tx_logs,
                    db_contract_creations,
                    db_contract_interactions,
                    db_token_transfers,
                )
                .await;

                let delete_receipts: Vec<String> = tx_receipts
                    .clone()
                    .into_iter()
                    .map(|receipt| format_hash(receipt.transaction_hash))
                    .collect();

                db.delete_no_receipt_txs(&delete_receipts).await;
            }
        });

        works.push(work);
    }

    join_all(works).await;

    Ok(())
}

fn vec_to_set(vec: Vec<i64>) -> HashSet<i64> {
    HashSet::from_iter(vec)
}
