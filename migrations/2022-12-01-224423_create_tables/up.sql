CREATE TABLE blocks (
  number BIGINT NOT NULL,
  hash VARCHAR NOT NULL PRIMARY KEY,
  difficulty  VARCHAR NOT NULL,
  total_difficulty  VARCHAR NOT NULL,
  miner VARCHAR NOT NULL,
  gas_limit VARCHAR NOT NULL,
  gas_used VARCHAR NOT NULL,
  txs BIGINT NOT NULL,
  timestamp VARCHAR NOT NULL,
  size VARCHAR NOT NULL,
  nonce VARCHAR NOT NULL,
  base_fee_per_gas VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);

CREATE TABLE txs_receipts (
  hash VARCHAR PRIMARY KEY UNIQUE,
  success BOOLEAN,
  chain VARCHAR NOT NULL
);

CREATE TABLE logs (
  hash_with_index VARCHAR PRIMARY KEY UNIQUE,
  hash VARCHAR,
  address VARCHAR,
  topics text[],
  data VARCHAR,
  log_index BIGINT,
  transaction_log_index BIGINT,
  log_type VARCHAR,
  chain VARCHAR NOT NULL
);

CREATE TABLE contract_interactions (
  hash VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  block BIGINT NOT NULL,
  address VARCHAR NOT NULL,
  contract VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);

CREATE TABLE contract_creations (
  hash VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  block BIGINT NOT NULL,
  contract VARCHAR NOT NULL,
  chain VARCHAR NOT NULL 
);

CREATE TABLE token_transfers (
  hash_with_index VARCHAR PRIMARY KEY UNIQUE NOT NULL, 
  hash VARCHAR NOT NULL, 
  log_index BIGINT NOT NULL,
  block BIGINT NOT NULL,
  token VARCHAR NOT NULL,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  value VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);

CREATE TABLE txs (
  hash VARCHAR PRIMARY KEY UNIQUE,
  block_number BIGINT NOT NULL,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  value VARCHAR NOT NULL,
  gas_used VARCHAR NOT NULL,
  gas_price VARCHAR NOT NULL,
  timestamp VARCHAR NOT NULL,
  transaction_index BIGINT NOT NULL,
  transaction_type BIGINT,
  max_fee_per_gas VARCHAR,
  max_priority_fee_per_gas VARCHAR,
  input VARCHAR NOT NULL,
  method_id VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);

CREATE TABLE tokens (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL, 
  name VARCHAR NOT NULL,
  decimals BIGINT NOT NULL,
  symbol VARCHAR NOT NULL
);

CREATE TABLE excluded_tokens (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);

CREATE TABLE txs_no_receipt (
  hash VARCHAR PRIMARY KEY UNIQUE,
  chain VARCHAR NOT NULL,
  block_number BIGINT NOT NULL
);

CREATE TABLE contract_abis (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  chain VARCHAR NOT NULL,
  address VARCHAR NOT NULL,
  abi VARCHAR,
  verified BOOLEAN
);

CREATE TABLE method_ids (
  method_id VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  name VARCHAR NOT NULL
);

INSERT INTO method_ids (method_id, name) VALUES ('0x00000000', 'Transfer');

CREATE TABLE contracts_adapters (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL,
  adapter_id VARCHAR NOT NULL
);