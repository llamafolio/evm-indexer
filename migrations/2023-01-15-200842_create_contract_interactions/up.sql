CREATE TABLE evm_contracts_interactions (
  hash TEXT PRIMARY KEY UNIQUE NOT NULL,
  block BIGINT NOT NULL,
  address TEXT NOT NULL,
  contract TEXT NOT NULL,
  chain TEXT NOT NULL
);