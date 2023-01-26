use dotenv::dotenv;
use ethabi::Contract;
use evm_indexer::chains::chains::{get_chain, ETHEREUM};
use evm_indexer::configs::abi_fetcher_config::EVMAbiFetcherConfig;
use evm_indexer::db::db::Database;
use evm_indexer::db::models::models::{DatabaseAbi, DatabaseContract, DatabaseMethod};
use log::LevelFilter;
use log::*;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Error;
use simple_logger::SimpleLogger;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiResponse {
    pub status: String,
    pub message: String,
    pub result: String,
}

#[tokio::main()]
async fn main() {
    dotenv().ok();

    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let config = EVMAbiFetcherConfig::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    info!("Starting EVM ABI fetcher");

    let db = Database::new(config.db_url, config.redis_url.clone(), ETHEREUM)
        .await
        .expect("Unable to start DB connection.");

    loop {
        let contracts = db.get_contracts().await.unwrap();

        if contracts.len() > 0 {
            info!("Fetching ABIs for {} contracts.", contracts.len());

            let client = Client::new();

            let mut contracts_fetched: Vec<DatabaseContract> = Vec::new();

            let mut abis_fetched: Vec<DatabaseAbi> = Vec::new();

            for mut contract in contracts {
                let uri_str: String;

                let chain = get_chain(contract.chain.clone());

                let token = config.api_source_tokens.get(chain.name);

                match token {
                    Some(token) => {
                        if chain.abi_source_require_auth {
                            uri_str = format!(
                                "{}api?module=contract&action=getabi&address={}&apikey={}",
                                chain.abi_source_api, contract.contract, token
                            );
                        } else {
                            uri_str = format!(
                                "{}api?module=contract&action=getabi&address={}",
                                chain.abi_source_api, contract.contract
                            );
                        }
                    }
                    None => {
                        continue;
                    }
                }

                let response = client.get(uri_str).send().await;

                match response {
                    Ok(data) => match data.text().await {
                        Ok(response) => {
                            let abi_response: Result<AbiResponse, Error> =
                                serde_json::from_str(&response);

                            let mut db_contract_abi = DatabaseAbi {
                                chain: chain.name.to_owned(),
                                contract: contract.contract.clone(),
                                abi: None,
                                verified: false,
                            };

                            match abi_response {
                                Ok(abi_response_formatted) => {
                                    if abi_response_formatted.status == "1"
                                        && abi_response_formatted.message == "OK"
                                    {
                                        db_contract_abi.abi = Some(abi_response_formatted.result);
                                        db_contract_abi.verified = true;

                                        contract.parsed = true;

                                        contracts_fetched.push(contract);
                                        abis_fetched.push(db_contract_abi);
                                    } else {
                                        if abi_response_formatted.result
                                            == "Contract source code not verified"
                                        {
                                            contract.parsed = true;

                                            contracts_fetched.push(contract);
                                            abis_fetched.push(db_contract_abi);
                                        }
                                    }
                                }
                                Err(_) => {
                                    continue;
                                }
                            }
                        }
                        Err(_) => {
                            continue;
                        }
                    },
                    Err(_) => {
                        continue;
                    }
                }
            }

            let mut methods: Vec<DatabaseMethod> = Vec::new();

            for abi in &abis_fetched {
                let contract: Contract = match &abi.abi {
                    Some(abi) => match serde_json::from_str(abi) {
                        Ok(contract) => contract,
                        Err(_) => {
                            continue;
                        }
                    },
                    None => {
                        continue;
                    }
                };

                let functions = contract.functions();

                for function in functions {
                    let signature = format!("0x{}", hex::encode(function.short_signature()));

                    let db_method = DatabaseMethod {
                        name: function.name.clone(),
                        method: signature,
                    };

                    methods.push(db_method);
                }
            }

            db.update_contracts(&contracts_fetched).await.unwrap();
            db.store_abis(&abis_fetched).await.unwrap();
            db.store_methods(&methods).await.unwrap();

            info!(
                "Stored {} ABIs from {} contracts with {} methods.",
                abis_fetched.len(),
                contracts_fetched.len(),
                methods.len()
            );
        }
    }
}
