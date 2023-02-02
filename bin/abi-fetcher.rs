use dotenv::dotenv;
use ethabi::Contract;
use evm_indexer::chains::chains::{get_chain, ETHEREUM};
use evm_indexer::configs::abi_fetcher_config::EVMAbiFetcherConfig;
use evm_indexer::db::db::Database;
use evm_indexer::db::models::models::{
    DatabaseContract, DatabaseContractInformation, DatabaseMethod,
};
use log::LevelFilter;
use log::*;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Error;
use simple_logger::SimpleLogger;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub status: String,
    pub message: String,
    pub result: Vec<ContractDataResult>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDataResult {
    #[serde(rename = "SourceCode")]
    pub source_code: String,
    #[serde(rename = "ABI")]
    pub abi: String,
    #[serde(rename = "ContractName")]
    pub contract_name: String,
    #[serde(rename = "CompilerVersion")]
    pub compiler_version: String,
    #[serde(rename = "OptimizationUsed")]
    pub optimization_used: String,
    #[serde(rename = "Runs")]
    pub runs: String,
    #[serde(rename = "ConstructorArguments")]
    pub constructor_arguments: String,
    #[serde(rename = "EVMVersion")]
    pub evmversion: String,
    #[serde(rename = "Library")]
    pub library: String,
    #[serde(rename = "LicenseType")]
    pub license_type: String,
    #[serde(rename = "Proxy")]
    pub proxy: String,
    #[serde(rename = "Implementation")]
    pub implementation: String,
    #[serde(rename = "SwarmSource")]
    pub swarm_source: String,
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

    let db = Database::new(config.db_url.clone(), config.redis_url.clone(), ETHEREUM)
        .await
        .expect("Unable to start DB connection.");

    loop {
        let contracts = db.get_contracts_missing_parsed().await.unwrap();

        if contracts.len() > 0 {
            info!("Fetching ABIs for {} contracts.", contracts.len());

            let client = Client::new();

            let mut contracts_fetched: Vec<DatabaseContract> = Vec::new();

            let mut contracts_information_fetched: Vec<DatabaseContractInformation> = Vec::new();

            for mut contract in contracts {
                let uri_str: String;

                let chain = get_chain(contract.chain.clone());

                let token = config.api_source_tokens.get(chain.name);

                match token {
                    Some(token) => {
                        if chain.abi_source_require_auth {
                            uri_str = format!(
                                "{}api?module=contract&action=getsourcecode&address={}&apikey={}",
                                chain.abi_source_api, contract.contract, token
                            );
                        } else {
                            uri_str = format!(
                                "{}api?module=contract&action=getsourcecode&address={}",
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
                            let contract_response: Result<Response, Error> =
                                serde_json::from_str(&response);

                            let mut db_contract_information = DatabaseContractInformation {
                                chain: chain.name.to_owned(),
                                contract: contract.contract.clone(),
                                abi: None,
                                name: None,
                                verified: false,
                            };

                            match contract_response {
                                Ok(contract_response_formatted) => {
                                    let result = match contract_response_formatted.result.first() {
                                        Some(result) => result,
                                        None => continue,
                                    };

                                    if contract_response_formatted.status == "1"
                                        && contract_response_formatted.message == "OK"
                                    {
                                        db_contract_information.abi = Some(result.abi.clone());
                                        db_contract_information.verified = true;

                                        contract.parsed = true;

                                        contracts_fetched.push(contract);

                                        contracts_information_fetched.push(db_contract_information);
                                    } else {
                                        if result.abi == "Contract source code not verified" {
                                            contract.parsed = true;

                                            contracts_fetched.push(contract);

                                            contracts_information_fetched
                                                .push(db_contract_information);
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

            for contracts_information in &contracts_information_fetched {
                let contract: Contract = match &contracts_information.abi {
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
            db.store_contracts_information(&contracts_information_fetched)
                .await
                .unwrap();
            db.store_methods(&methods).await.unwrap();

            info!(
                "Stored {} ABIs from {} contracts with {} methods.",
                contracts_information_fetched.len(),
                contracts_fetched.len(),
                methods.len()
            );
        }
    }
}
