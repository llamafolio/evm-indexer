use std::{collections::HashSet, time::Duration};

use anyhow::Result;
use ethabi::Contract;
use log::*;
use reqwest::Client;
use serde_json::Error;
use tokio::time::sleep;
use web3::futures::future::join_all;

use crate::{
    config::Config,
    db::{
        models::{
            DatabaseBlock, DatabaseContractABI, DatabaseContractAdapter, DatabaseContractCreation,
            DatabaseContractInteraction, DatabaseExcludedToken, DatabaseMethodID, DatabaseToken,
            DatabaseTokenTransfers, DatabaseTx, DatabaseTxLogs, DatabaseTxNoReceipt,
            DatabaseTxReceipt,
        },
        Database,
    },
    rpc::Rpc,
};

use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiResponse {
    pub status: String,
    pub message: String,
    pub result: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterIDsResponse {
    pub data: Vec<AdapterID>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterID {
    pub adapter_id: String,
    pub address: String,
}

pub async fn fetch_blocks_singles(db: &Database, config: &Config, rpc: &Rpc) -> Result<()> {
    let rpc_last_block = rpc.get_last_block().await.unwrap();

    let full_blocks_set: Vec<i64> = (1..rpc_last_block).collect();

    let db_blocks_set = vec_to_set(db.get_block_numbers().await.unwrap());

    let missing_blocks: Vec<i64> = full_blocks_set
        .into_iter()
        .filter(|n| !db_blocks_set.contains(n))
        .collect();

    let missing_blocks_amount = missing_blocks.len();

    info!(
        "Fetching {} blocks with batches of {} blocks",
        missing_blocks_amount, config.batch_size
    );

    let single_fetch_worker_batch = missing_blocks.chunks(config.workers);

    let mut works = vec![];
    for worker_chunk in single_fetch_worker_batch {
        let worker = tokio::spawn({
            let config = config.clone();
            let rpc = rpc.clone();
            let db = db.clone();
            let chunk = worker_chunk.to_vec();

            async move {
                for block in chunk {
                    let (
                        db_block,
                        mut db_txs,
                        db_tx_receipts,
                        db_tx_logs,
                        db_contract_creation,
                        db_contract_interaction,
                        db_token_transfers,
                    ) = rpc.get_block(&config, block.clone()).await.unwrap();

                    let db_txs_count = db_txs.len();
                    let db_tx_receipts_count = db_tx_receipts.len();

                    let mut enough_receipts = true;

                    if db_txs_count != db_tx_receipts_count {
                        info!(
                            "Not enough receipts for batch: txs({}) receipts ({})",
                            db_txs_count, db_tx_receipts_count,
                        );

                        enough_receipts = false;
                    }

                    if !enough_receipts {
                        let db_receipts_hash: HashSet<String> = vec_string_to_set(
                            db_tx_receipts
                                .clone()
                                .into_iter()
                                .map(|receipt| receipt.hash)
                                .collect(),
                        );

                        let mut db_txs_with_no_receipts: Vec<DatabaseTxNoReceipt> = vec![];

                        for tx in &mut db_txs {
                            let hash = tx.hash.clone();
                            let chain = tx.chain.clone();
                            if !db_receipts_hash.contains(&hash) {
                                db_txs_with_no_receipts.push(DatabaseTxNoReceipt {
                                    hash,
                                    chain,
                                    block_number: tx.block_number,
                                });
                            }
                        }

                        info!(
                            "Storing {} txs with no receipt for future check",
                            db_txs_with_no_receipts.len(),
                        );

                        db.store_txs_no_receipt(&db_txs_with_no_receipts).await;
                    }

                    db.store_blocks_and_txs(
                        vec![db_block],
                        db_txs,
                        db_tx_receipts,
                        db_tx_logs,
                        db_contract_creation,
                        db_contract_interaction,
                        db_token_transfers,
                    )
                    .await;
                }
            }
        });
        works.push(worker)
    }

    join_all(works).await;
    Ok(())
}

pub async fn fetch_blocks(db: &Database, config: &Config, rpc: &Rpc) -> Result<()> {
    let rpc_last_block = rpc.get_last_block().await.unwrap();

    let full_blocks_set: Vec<i64> = (1..rpc_last_block).collect();

    let db_blocks_set = vec_to_set(db.get_block_numbers().await.unwrap());

    let missing_blocks: Vec<i64> = full_blocks_set
        .into_iter()
        .filter(|n| !db_blocks_set.contains(n))
        .collect();

    let missing_blocks_amount = missing_blocks.len();

    info!(
        "Fetching {} blocks with batches of {} blocks",
        missing_blocks_amount, config.batch_size
    );

    let chunks: Vec<Vec<i64>> = missing_blocks
        .clone()
        .chunks(config.batch_size * config.workers)
        .map(|chunk| chunk.to_vec())
        .collect();

    for chunk in chunks {
        info!(
            "Procesing chunk from block {} to {} for chain {}",
            chunk.first().unwrap(),
            chunk.last().unwrap(),
            config.chain.name
        );

        let mut works = vec![];

        let worker_chunks: Vec<Vec<i64>> = chunk
            .chunks(config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut db_blocks: Vec<DatabaseBlock> = Vec::new();
        let mut db_txs: Vec<DatabaseTx> = Vec::new();
        let mut db_tx_receipts: Vec<DatabaseTxReceipt> = Vec::new();
        let mut db_tx_logs: Vec<DatabaseTxLogs> = Vec::new();
        let mut db_contract_creation: Vec<DatabaseContractCreation> = Vec::new();
        let mut db_contract_interaction: Vec<DatabaseContractInteraction> = Vec::new();
        let mut db_token_transfers: Vec<DatabaseTokenTransfers> = Vec::new();

        for worker_chunk in worker_chunks {
            let worker = tokio::spawn({
                let chunk = worker_chunk.clone();
                let config = config.clone();
                let rpc = rpc.clone();
                async move {
                    return rpc.get_blocks(&config, chunk.to_vec()).await.unwrap();
                }
            });
            works.push(worker);
        }

        let results = join_all(works).await;

        results.into_iter().for_each(|result| {
            let (
                mut blocks,
                mut txs,
                mut receipts,
                mut logs,
                mut contract_creations,
                mut contract_interactions,
                mut token_transfers,
            ) = result.unwrap();

            db_blocks.append(&mut blocks);
            db_txs.append(&mut txs);
            db_tx_receipts.append(&mut receipts);
            db_tx_logs.append(&mut logs);
            db_contract_creation.append(&mut contract_creations);
            db_contract_interaction.append(&mut contract_interactions);
            db_token_transfers.append(&mut token_transfers);
        });

        let db_txs_count = db_txs.len();
        let db_tx_receipts_count = db_tx_receipts.len();

        let mut enough_receipts = true;

        if db_blocks.len() > 0 {
            if db_txs_count != db_tx_receipts_count {
                info!(
                    "Not enough receipts for batch: txs({}) receipts ({}) block_range({})-({})",
                    db_txs_count,
                    db_tx_receipts_count,
                    db_blocks.first().unwrap().number,
                    db_blocks.last().unwrap().number,
                );

                enough_receipts = false;
            }

            if !enough_receipts {
                let db_receipts_hash: HashSet<String> = vec_string_to_set(
                    db_tx_receipts
                        .clone()
                        .into_iter()
                        .map(|receipt| receipt.hash)
                        .collect(),
                );

                let mut db_txs_with_no_receipts: Vec<DatabaseTxNoReceipt> = vec![];

                for tx in &mut db_txs {
                    let hash = tx.hash.clone();
                    let chain = tx.chain.clone();
                    if !db_receipts_hash.contains(&hash) {
                        db_txs_with_no_receipts.push(DatabaseTxNoReceipt {
                            hash,
                            chain,
                            block_number: tx.block_number,
                        });
                    }
                }

                info!(
                    "Storing {} txs with no receipt for future check",
                    db_txs_with_no_receipts.len(),
                );

                db.store_txs_no_receipt(&db_txs_with_no_receipts).await;
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
    }

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

    let mut chunk_size = 1000;

    if config.local_rpc_http != String::from("") {
        chunk_size = 200;
    }

    let chunks: Vec<Vec<String>> = missing_txs
        .chunks(chunk_size)
        .into_iter()
        .map(|chunk| chunk.to_vec())
        .collect();

    let mut works = vec![];

    for chunk in chunks {
        let work = tokio::spawn({
            let work_chunk = chunk.clone();

            let rpc = rpc.clone();

            async move {
                let tx_receipts = rpc.get_txs_receipts(&work_chunk.to_vec()).await.unwrap();

                if tx_receipts.len() == 0 {
                    return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
                }

                return rpc.get_metadata_from_receipts(&tx_receipts).await.unwrap();
            }
        });

        works.push(work);
    }

    let results = join_all(works).await;

    let mut db_tx_receipts: Vec<DatabaseTxReceipt> = Vec::new();
    let mut db_tx_logs: Vec<DatabaseTxLogs> = Vec::new();
    let mut db_contract_creations: Vec<DatabaseContractCreation> = Vec::new();
    let mut db_token_transfers: Vec<DatabaseTokenTransfers> = Vec::new();

    results.into_iter().for_each(|result| {
        let (mut receipts, mut logs, mut contract_creations, mut token_transfers) = result.unwrap();

        db_tx_receipts.append(&mut receipts);
        db_tx_logs.append(&mut logs);
        db_contract_creations.append(&mut contract_creations);
        db_token_transfers.append(&mut token_transfers);
    });

    let delete_receipts: Vec<String> = db_tx_receipts
        .clone()
        .into_iter()
        .map(|receipt| receipt.hash.clone())
        .collect();

    db.store_blocks_and_txs(
        Vec::new(),
        Vec::new(),
        db_tx_receipts,
        db_tx_logs,
        db_contract_creations,
        Vec::new(),
        db_token_transfers,
    )
    .await;

    db.delete_no_receipt_txs(&delete_receipts).await;

    Ok(())
}

pub async fn fetch_contract_abis(config: &Config, db: &Database, token: &str) -> Result<()> {
    let contracts = db.get_created_contracts().await.unwrap();
    let contracts_with_abis = db.get_contracts_with_abis().await.unwrap();
    let already_fetched_contracts_set = vec_string_to_set(contracts_with_abis);

    let pending_contracts: Vec<String> = contracts
        .into_iter()
        .filter(|contract| !already_fetched_contracts_set.contains(contract))
        .collect();

    let pending_contracts_amount = pending_contracts.len();

    info!("Fetching ABIs for {} contracts", pending_contracts_amount);

    let client = Client::new();

    for pending_contract in pending_contracts {
        let uri_str: String;

        if config.chain.abi_source_require_auth {
            uri_str = format!(
                "{}api?module=contract&action=getabi&address={}&apikey={}",
                config.chain.abi_source_url, pending_contract, token
            );
        } else {
            uri_str = format!(
                "{}api?module=contract&action=getabi&address={}",
                config.chain.abi_source_url, pending_contract
            );
        }

        let response = client.get(uri_str).send().await;

        match response {
            Ok(data) => match data.text().await {
                Ok(response) => {
                    let abi_response: Result<AbiResponse, Error> = serde_json::from_str(&response);
                    match abi_response {
                        Ok(abi_response_formatted) => {
                            if abi_response_formatted.status == "1"
                                && abi_response_formatted.message == "OK"
                            {
                                let abi = abi_response_formatted.result;

                                let db_contract_abi = DatabaseContractABI {
                                    address_with_chain: format!(
                                        "{}-{}",
                                        pending_contract,
                                        config.chain.name.clone()
                                    ),
                                    chain: config.chain.name.to_string(),
                                    address: pending_contract,
                                    abi: Some(abi.clone()),
                                    verified: true,
                                };

                                db.store_contract_abi(&db_contract_abi).await;

                                let contract: Contract = serde_json::from_str(&abi).unwrap();

                                let functions = contract.functions();

                                let mut db_methods: Vec<DatabaseMethodID> = vec![];

                                for function in functions {
                                    let signature =
                                        format!("0x{}", hex::encode(function.short_signature()));

                                    let db_method = DatabaseMethodID {
                                        name: function.name.clone(),
                                        method_id: signature,
                                    };

                                    db_methods.push(db_method);
                                }

                                info!("Storing {} method IDs from ABI", db_methods.len());

                                db.store_abi_method_ids(&db_methods).await
                            } else {
                                if abi_response_formatted.result
                                    == "Contract source code not verified"
                                {
                                    let db_contract_abi = DatabaseContractABI {
                                        address_with_chain: format!(
                                            "{}-{}",
                                            pending_contract,
                                            config.chain.name.clone()
                                        ),
                                        chain: config.chain.name.to_string(),
                                        address: pending_contract,
                                        abi: None,
                                        verified: false,
                                    };

                                    db.store_contract_abi(&db_contract_abi).await;
                                }
                            }
                        }
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }

    Ok(())
}

pub async fn fetch_adapters(config: &Config, db: &Database) -> Result<()> {
    let mut chainname = config.chain.name;

    if chainname == "mainnet" {
        chainname = "ethereum";
    }

    info!("Fetching adapter IDs for {}", chainname);

    let uri = format!(
        "https://rifcoe52qb.execute-api.eu-central-1.amazonaws.com/adapters/{}",
        chainname
    );

    let client = Client::new();

    let response = client.get(uri).send().await;

    match response {
        Ok(data) => match data.text().await {
            Ok(response) => {
                let adapter_ids: Result<AdapterIDsResponse, Error> =
                    serde_json::from_str(&response);

                match adapter_ids {
                    Ok(adapter_ids) => {
                        let contract_adapters: Vec<DatabaseContractAdapter> = adapter_ids
                            .data
                            .into_iter()
                            .map(|contract_adapter| DatabaseContractAdapter {
                                address_with_chain: format!(
                                    "{}-{}",
                                    contract_adapter.address,
                                    config.chain.name.clone()
                                ),
                                adapter_id: contract_adapter.adapter_id,
                                chain: config.chain.name.to_string(),
                                address: contract_adapter.address,
                            })
                            .collect();

                        info!("Storing {} adapter ids", contract_adapters.len());

                        db.store_contract_adapters(&contract_adapters).await;

                        Ok(())
                    }
                    Err(_) => Ok(()),
                }
            }
            Err(_) => Ok(()),
        },
        Err(_) => Ok(()),
    }
}

fn vec_to_set(vec: Vec<i64>) -> HashSet<i64> {
    HashSet::from_iter(vec)
}

fn vec_string_to_set(vec: Vec<String>) -> HashSet<String> {
    HashSet::from_iter(vec)
}
