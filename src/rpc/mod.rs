use anyhow::Result;
use ethabi::ParamType;
use log::*;
use web3::{
    futures::StreamExt,
    transports::{Batch, Http, WebSocket},
    types::{Block, BlockId, Log, Transaction, TransactionReceipt, U64},
    Web3,
};

use crate::{
    config::Config,
    db::{
        models::{
            DatabaseBlock, DatabaseContractCreation, DatabaseContractInteraction,
            DatabaseTokenTransfers, DatabaseTx, DatabaseTxLogs, DatabaseTxReceipt,
        },
        Database,
    },
    utils::{format_address, format_block, format_bool, format_hash, format_receipt},
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub batch: Web3<Batch<Http>>,
    pub wss: Web3<WebSocket>,
    pub chain: String,
}

impl Rpc {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing RPC");

        let http = Http::new(&config.rpc_http_url).unwrap();
        let ws = WebSocket::new(&config.rpc_ws_url).await.unwrap();

        Ok(Self {
            wss: Web3::new(ws),
            batch: Web3::new(web3::transports::Batch::new(http)),
            chain: config.chain,
        })
    }

    pub async fn get_last_block(&self) -> Result<i64> {
        let last_block = self
            .wss
            .eth()
            .block_number()
            .await
            .expect("Unable to fetch last block")
            .as_u64();

        Ok(last_block as i64)
    }

    async fn get_block_batch(&self, range: Vec<i64>) -> Result<Vec<Block<Transaction>>> {
        for block_height in range.iter() {
            let block_number = U64::from_str_radix(&block_height.to_string(), 10)
                .expect("Unable to parse block number");

            let block_id = <BlockId as From<U64>>::from(block_number);

            self.batch.eth().block_with_txs(block_id);
        }

        let blocks_res = self.batch.transport().submit_batch().await;

        match blocks_res {
            Ok(result) => Ok(result
                .into_iter()
                .map(|block| format_block(&block))
                .collect()),
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn get_txs_receipts(&self, txs: &Vec<Transaction>) -> Result<Vec<TransactionReceipt>> {
        for tx in txs.iter() {
            self.batch.eth().transaction_receipt(tx.hash);
        }
        let result = self.batch.transport().submit_batch().await;

        match result {
            Ok(result) => Ok(result
                .into_iter()
                .map(|receipt| format_receipt(&receipt))
                .collect()),
            Err(_) => Ok(Vec::new()),
        }
    }

    pub async fn subscribe_heads(&self, db: &Database) {
        let mut sub = self
            .wss
            .eth_subscribe()
            .subscribe_new_heads()
            .await
            .unwrap();

        info!("Initializing new blocks listener");

        loop {
            let new_block = sub.next().await;

            match new_block {
                Some(block_header) => match block_header {
                    Ok(block_header) => {
                        let block_number = block_header.number.unwrap();
                        info!(
                            "Received new block header with height {:?}",
                            block_header.number.unwrap()
                        );

                        let from = block_number.as_u64() as i64 - 5;
                        let to = block_number.as_u64() as i64;

                        let range: Vec<i64> = (from..to).collect();

                        let (
                            db_blocks,
                            db_txs,
                            db_tx_receipts,
                            db_tx_logs,
                            db_contract_creations,
                            db_contract_interactions,
                            db_token_transfers,
                        ) = self.get_blocks(range).await.unwrap();

                        db.store_blocks_and_txs(
                            db_blocks,
                            db_txs,
                            db_tx_receipts,
                            db_tx_logs,
                            db_contract_creations,
                            db_contract_interactions,
                            db_token_transfers,
                        )
                        .await;
                    }
                    Err(_) => {
                        continue;
                    }
                },
                None => {
                    continue;
                }
            }
        }
    }

    pub async fn get_blocks(
        &self,
        range: Vec<i64>,
    ) -> Result<(
        Vec<DatabaseBlock>,
        Vec<DatabaseTx>,
        Vec<DatabaseTxReceipt>,
        Vec<DatabaseTxLogs>,
        Vec<DatabaseContractCreation>,
        Vec<DatabaseContractInteraction>,
        Vec<DatabaseTokenTransfers>,
    )> {
        let blocks = self.get_block_batch(range).await.unwrap();

        let (db_blocks, web3_vec_txs): (Vec<DatabaseBlock>, Vec<Vec<Transaction>>) = blocks
            .into_iter()
            .map(|block| {
                (
                    DatabaseBlock::from_web3(&block, self.chain.clone()),
                    block.transactions,
                )
            })
            .unzip();

        let web3_txs: Vec<Transaction> = web3_vec_txs.into_iter().flatten().collect();

        let web3_receipts = self.get_txs_receipts(&web3_txs).await.into_iter().flatten();

        let db_txs: Vec<DatabaseTx> = web3_txs
            .into_iter()
            .map(|tx| DatabaseTx::from_web3(&tx, self.chain.clone()))
            .collect();

        let mut db_tx_receipts: Vec<DatabaseTxReceipt> = vec![];

        let mut web3_vec_tx_logs: Vec<Vec<Log>> = vec![];

        let mut db_contract_creations: Vec<DatabaseContractCreation> = vec![];

        let mut db_contract_interactions: Vec<DatabaseContractInteraction> = vec![];

        let mut db_token_transfers: Vec<DatabaseTokenTransfers> = vec![];

        for tx_receipt in web3_receipts {
            let db_tx_receipt =
                DatabaseTxReceipt::from_web3(tx_receipt.clone(), self.chain.clone());

            db_tx_receipts.push(db_tx_receipt);

            let success: bool = match tx_receipt.status {
                None => false,
                Some(success) => format_bool(success),
            };

            if success {
                match tx_receipt.contract_address {
                    Some(contract) => db_contract_creations.push(DatabaseContractCreation {
                        hash: format_hash(tx_receipt.transaction_hash),
                        block: tx_receipt.block_number.unwrap().as_u64() as i64,
                        contract: format_address(contract),
                        chain: self.chain.clone(),
                    }),
                    None => {
                        if tx_receipt.logs.len() > 0 {
                            db_contract_interactions.push(DatabaseContractInteraction {
                                hash: format_hash(tx_receipt.transaction_hash),
                                block: tx_receipt.block_number.unwrap().as_u64() as i64,
                                address: format_address(tx_receipt.from),
                                contract: format_address(tx_receipt.to.unwrap()),
                                chain: self.chain.clone(),
                            });

                            // Check for token transfers
                            for log in tx_receipt.logs.clone() {
                                if log.topics.len() != 3 {
                                    continue;
                                }

                                let event = ethabi::Event {
                                    name: "Transfer".to_owned(),
                                    inputs: vec![
                                        ethabi::EventParam {
                                            name: "from".to_owned(),
                                            kind: ParamType::Address,
                                            indexed: false,
                                        },
                                        ethabi::EventParam {
                                            name: "to".to_owned(),
                                            kind: ParamType::Address,
                                            indexed: false,
                                        },
                                        ethabi::EventParam {
                                            name: "amount".to_owned(),
                                            kind: ParamType::Uint(256),
                                            indexed: false,
                                        },
                                    ],
                                    anonymous: false,
                                };

                                // Check the first topic against keccak256(Transfer(address,address,uint256))
                                if format_hash(log.topics[0]) != format!("{:?}", event.signature())
                                {
                                    continue;
                                }

                                let from_address: String = match ethabi::decode(
                                    &[ParamType::Address],
                                    log.topics[1].as_bytes(),
                                ) {
                                    Ok(address) => {
                                        if address.len() == 0 {
                                            continue;
                                        } else {
                                            format!(
                                                "{:?}",
                                                address[0].clone().into_address().unwrap()
                                            )
                                        }
                                    }
                                    Err(_) => continue,
                                };

                                let to_address = match ethabi::decode(
                                    &[ParamType::Address],
                                    log.topics[2].as_bytes(),
                                ) {
                                    Ok(address) => {
                                        if address.len() == 0 {
                                            continue;
                                        } else {
                                            format!(
                                                "{:?}",
                                                address[0].clone().into_address().unwrap()
                                            )
                                        }
                                    }
                                    Err(_) => continue,
                                };

                                let value = match ethabi::decode(
                                    &[ParamType::Uint(256)],
                                    &log.data.0[..],
                                ) {
                                    Ok(value) => {
                                        if value.len() == 0 {
                                            continue;
                                        } else {
                                            format!("{:?}", value[0].clone().into_uint().unwrap())
                                        }
                                    }
                                    Err(_) => continue,
                                };

                                db_token_transfers.push(DatabaseTokenTransfers {
                                    hash_with_index: format!(
                                        "{}-{}",
                                        format_hash(tx_receipt.transaction_hash),
                                        log.log_index.unwrap().as_u64()
                                    ),
                                    block: tx_receipt.block_number.unwrap().as_u64() as i64,
                                    token: format_address(log.address),
                                    from_address,
                                    to_address,
                                    value,
                                    chain: self.chain.clone(),
                                })
                            }
                        }
                    }
                }
            }

            web3_vec_tx_logs.push(tx_receipt.logs);
        }

        let db_tx_logs: Vec<DatabaseTxLogs> = web3_vec_tx_logs
            .into_iter()
            .flatten()
            .map(|log| DatabaseTxLogs::from_web3(log, self.chain.clone()))
            .collect();

        Ok((
            db_blocks,
            db_txs,
            db_tx_receipts,
            db_tx_logs,
            db_contract_creations,
            db_contract_interactions,
            db_token_transfers,
        ))
    }
}
