CREATE INDEX IF NOT EXISTS txs_by_chain
ON txs (chain);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_adapter_id
ON contracts_adapters (adapter_id);

CREATE INDEX IF NOT EXISTS contract_interactions_by_contract
ON contract_interactions (contract);