CREATE TABLE contracts_adapters (
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL,
  adapter_id VARCHAR NOT NULL,
  PRIMARY KEY (address, chain)
);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_address
ON contracts_adapters (address);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_adapter_id
ON contracts_adapters (adapter_id);