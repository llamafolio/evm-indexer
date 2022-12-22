CREATE INDEX IF NOT EXISTS token_transfers_by_hash 
ON token_transfers (hash);

CREATE INDEX IF NOT EXISTS token_transfers_by_sender
ON token_transfers (from_address);

CREATE INDEX IF NOT EXISTS token_transfers_by_receiver
ON token_transfers (to_address);

CREATE INDEX IF NOT EXISTS tokens_by_chain 
ON tokens (chain);

CREATE INDEX IF NOT EXISTS tokens_by_address 
ON tokens (address);

CREATE INDEX IF NOT EXISTS txs_by_block_number 
ON txs (block_number DESC);

CREATE INDEX IF NOT EXISTS txs_by_sender 
ON txs (from_address);

CREATE INDEX IF NOT EXISTS txs_by_receiver
ON txs (to_address);

CREATE INDEX IF NOT EXISTS contract_interactions_by_address
ON contract_interactions (address);

CREATE INDEX IF NOT EXISTS tx_logs_by_hash
ON logs (hash);

CREATE INDEX IF NOT EXISTS contract_abis_by_address
ON contract_abis (address);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_address
ON contracts_adapters (address);

CREATE INDEX IF NOT EXISTS txs_by_chain
ON txs (chain);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_adapter_id
ON contracts_adapters (adapter_id);

CREATE INDEX IF NOT EXISTS contract_interactions_by_contract
ON contract_interactions (contract);

CREATE INDEX IF NOT EXISTS txs_by_timestamp
ON txs (timestamp DESC);