CREATE TABLE contracts_adapters (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL,
  adapter_id VARCHAR NOT NULL
);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_address
ON contracts_adapters (address);