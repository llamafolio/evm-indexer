use crate::{
    chains::evm_chains::Chain,
    configs::indexer_config::EVMIndexerConfig,
    db::models::models::{
        DatabaseEVMBlock, DatabaseEVMContract, DatabaseEVMTransaction, DatabaseEVMTransactionLog,
        DatabaseEVMTransactionReceipt,
    },
};
use ethers::types::{Block, Transaction, TransactionReceipt, U256};

use anyhow::Result;
use jsonrpsee::core::{client::ClientT, rpc_params};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use log::info;
use rand::seq::SliceRandom;
use std::time::Duration;

use serde_json::Error;

#[derive(Debug, Clone)]
pub struct EVMRpc {
    pub clients: Vec<HttpClient>,
    pub chain: Chain,
}

impl EVMRpc {
    pub async fn new(config: &EVMIndexerConfig) -> Result<Self> {
        info!("Starting EVM rpc service");

        let timeout = Duration::from_secs(60);

        let mut clients = Vec::new();

        for rpc in config.rpcs.clone() {
            let client = HttpClientBuilder::default()
                .max_concurrent_requests(100000)
                .request_timeout(timeout)
                .build(rpc)
                .unwrap();

            let client_id = client.request("eth_chainId", rpc_params![]).await;

            match client_id {
                Ok(value) => {
                    let chain_id: U256 = match serde_json::from_value(value) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };

                    if chain_id.as_u64() as i64 != config.chain.id {
                        continue;
                    }

                    clients.push(client);
                }
                Err(_) => continue,
            }
        }

        if clients.len() == 0 {
            panic!("No valid RPC client found");
        }

        Ok(Self {
            clients,
            chain: config.chain,
        })
    }

    pub async fn get_last_block(&self) -> Result<i64> {
        let client = self.get_client();

        let last_block = client.request("eth_blockNumber", rpc_params![]).await;

        match last_block {
            Ok(value) => {
                let block_number: U256 = serde_json::from_value(value)
                    .expect("Unable to deserialize eth_blockNumber response");

                Ok(block_number.as_u64() as i64)
            }
            Err(_) => Ok(0),
        }
    }

    pub async fn get_block(
        &self,
        block_number: &i64,
    ) -> Result<Option<(DatabaseEVMBlock, Vec<DatabaseEVMTransaction>)>> {
        let client = self.get_client();

        let raw_block = client
            .request(
                "eth_getBlockByNumber",
                rpc_params![format!("0x{:x}", block_number), true],
            )
            .await;

        match raw_block {
            Ok(value) => {
                let block: Result<Block<Transaction>, Error> = serde_json::from_value(value);

                match block {
                    Ok(block) => {
                        let db_block = DatabaseEVMBlock::from_rpc(&block, self.chain.name);

                        let mut db_transactions = Vec::new();

                        for transaction in block.transactions {
                            let db_transaction = DatabaseEVMTransaction::from_rpc(
                                transaction,
                                self.chain.name,
                                db_block.timestamp.clone(),
                            );

                            db_transactions.push(db_transaction)
                        }

                        Ok(Some((db_block, db_transactions)))
                    }
                    Err(_) => Ok(None),
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn get_transaction_receipt(
        &self,
        transaction: String,
    ) -> Result<
        Option<(
            DatabaseEVMTransactionReceipt,
            Vec<DatabaseEVMTransactionLog>,
            Option<DatabaseEVMContract>,
        )>,
    > {
        let client = self.get_client();

        let raw_receipt = client
            .request("eth_getTransactionReceipt", rpc_params![transaction])
            .await;

        match raw_receipt {
            Ok(value) => {
                let receipt: Result<TransactionReceipt, Error> = serde_json::from_value(value);

                match receipt {
                    Ok(receipt) => {
                        let db_receipt = DatabaseEVMTransactionReceipt::from_rpc(&receipt);

                        let mut db_transaction_logs: Vec<DatabaseEVMTransactionLog> = Vec::new();

                        let db_contract = match receipt.contract_address {
                            Some(_) => Some(DatabaseEVMContract::from_rpc(
                                receipt.clone(),
                                self.chain.name,
                            )),
                            None => None,
                        };

                        for log in receipt.logs {
                            let db_log = DatabaseEVMTransactionLog::from_rpc(log);

                            db_transaction_logs.push(db_log)
                        }

                        return Ok(Some((db_receipt, db_transaction_logs, db_contract)));
                    }
                    Err(_) => return Ok(None),
                }
            }
            Err(_) => return Ok(None),
        }
    }

    pub async fn get_block_receipts(
        &self,
        block_number: &i64,
    ) -> Result<
        Option<(
            Vec<DatabaseEVMTransactionReceipt>,
            Vec<DatabaseEVMTransactionLog>,
            Vec<DatabaseEVMContract>,
        )>,
    > {
        let client = self.get_client();

        let raw_receipts = client
            .request(
                "eth_getBlockReceipts",
                rpc_params![format!("0x{:x}", block_number)],
            )
            .await;

        match raw_receipts {
            Ok(value) => {
                let receipts: Result<Vec<TransactionReceipt>, Error> =
                    serde_json::from_value(value);

                match receipts {
                    Ok(receipts) => {
                        let mut db_receipts: Vec<DatabaseEVMTransactionReceipt> = Vec::new();

                        let mut db_transaction_logs: Vec<DatabaseEVMTransactionLog> = Vec::new();

                        let mut db_contracts: Vec<DatabaseEVMContract> = Vec::new();

                        for receipt in receipts {
                            let db_receipt = DatabaseEVMTransactionReceipt::from_rpc(&receipt);

                            db_receipts.push(db_receipt);

                            let db_contract = match receipt.contract_address {
                                Some(_) => Some(DatabaseEVMContract::from_rpc(
                                    receipt.clone(),
                                    self.chain.name,
                                )),
                                None => None,
                            };

                            if db_contract.is_some() {
                                db_contracts.push(db_contract.unwrap())
                            }

                            for log in receipt.logs {
                                let db_log = DatabaseEVMTransactionLog::from_rpc(log);

                                db_transaction_logs.push(db_log)
                            }
                        }

                        return Ok(Some((db_receipts, db_transaction_logs, db_contracts)));
                    }
                    Err(_) => return Ok(None),
                }
            }
            Err(_) => return Ok(None),
        }
    }

    fn get_client(&self) -> &HttpClient {
        let client = self.clients.choose(&mut rand::thread_rng()).unwrap();
        return client;
    }
}
