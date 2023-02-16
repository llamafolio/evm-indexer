use crate::{
    chains::chains::get_chains,
    db::db::{get_chunks, Database},
};
use anyhow::Result;
use field_count::FieldCount;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, FieldCount)]
pub struct DatabaseContractAdapter {
    pub adapter_id: String,
    pub chain: String,
    pub address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterID {
    pub adapter_id: String,
    pub address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterIDsResponse {
    pub data: Vec<AdapterID>,
}

pub struct LlamafolioParser {}

impl LlamafolioParser {
    pub async fn fetch(&self) -> Result<Vec<DatabaseContractAdapter>> {
        let mut adapters = Vec::new();

        for (chainname, _) in get_chains() {
            info!("Fetching adapter IDs for {}", chainname);

            let uri = format!(
                "https://js3czchveb.execute-api.eu-central-1.amazonaws.com/adapters/{}",
                chainname
            );

            let client = Client::new();

            let response = client.get(uri).send().await;

            match response {
                Ok(data) => match data.text().await {
                    Ok(response) => {
                        let adapter_ids = serde_json::from_str::<AdapterIDsResponse>(&response);

                        match adapter_ids {
                            Ok(adapter_ids) => {
                                let mut contract_adapters: Vec<DatabaseContractAdapter> =
                                    adapter_ids
                                        .data
                                        .into_iter()
                                        .map(|contract_adapter| DatabaseContractAdapter {
                                            adapter_id: contract_adapter.adapter_id,
                                            chain: chainname.to_string(),
                                            address: contract_adapter.address,
                                        })
                                        .collect();

                                adapters.append(&mut contract_adapters);
                            }
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                },
                Err(_) => continue,
            }
        }

        Ok(adapters)
    }

    pub async fn parse(
        &self,
        db: &Database,
        adapters: &Vec<DatabaseContractAdapter>,
    ) -> Result<()> {
        let mut connection = db.establish_connection().await;

        let chunks = get_chunks(adapters.len(), DatabaseContractAdapter::field_count());

        for (start, end) in chunks {
            diesel::insert_into(contracts_adapters::dsl::contracts_adapters)
                .values(&adapters[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("Unable to store contract adapters into database");
        }

        info!(
            "Inserted {} contract adapters to the database.",
            adapters.len()
        );

        Ok(())
    }
}
