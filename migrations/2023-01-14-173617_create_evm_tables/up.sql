CREATE TABLE evm_blocks (
  base_fee_per_gas TEXT NOT NULL,
  chain TEXT NOT NULL, 
  difficulty TEXT NOT NULL, 
  extra_data TEXT NOT NULL, 
  gas_limit TEXT NOT NULL, 
  gas_used TEXT NOT NULL, 
  block_hash TEXT PRIMARY KEY NOT NULL,
  logs_bloom TEXT NOT NULL, 
  miner TEXT NOT NULL, 
  mix_hash TEXT NOT NULL, 
  nonce TEXT NOT NULL, 
  number BIGINT NOT NULL, 
  parent_hash TEXT NOT NULL, 
  receipts_root TEXT NOT NULL, 
  sha3_uncles TEXT NOT NULL, 
  size BIGINT NOT NULL, 
  state_root TEXT NOT NULL, 
  timestamp TEXT NOT NULL, 
  total_difficulty TEXT NOT NULL, 
  transactions BIGINT NOT NULL,
  uncles TEXT[] NOT NULL
);

CREATE TABLE evm_methods (
  method TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL
);

CREATE TABLE evm_abis (
  chain TEXT NOT NULL,
  contract TEXT NOT NULL,
  abi TEXT,
  verified BOOLEAN NOT NULL,
  PRIMARY KEY (contract, chain)
);

CREATE TABLE evm_transactions (
  block_hash TEXT NOT NULL,
  block_number BIGINT NOT NULL, 
  chain TEXT NOT NULL,
  from_address TEXT NOT NULL, 
  gas TEXT NOT NULL, 
  gas_price TEXT NOT NULL, 
  max_priority_fee_per_gas TEXT, 
  max_fee_per_gas TEXT, 
  hash TEXT PRIMARY KEY NOT NULL,  
  input TEXT NOT NULL, 
  method TEXT NOT NULL,
  nonce TEXT NOT NULL, 
  timestamp TEXT NOT NULL, 
  to_address TEXT NOT NULL, 
  transaction_index BIGINT NOT NULL, 
  transaction_type BIGINT, 
  value TEXT NOT NULL
);

CREATE TABLE evm_contracts (
  block BIGINT NOT NULL,
  chain TEXT NOT NULL, 
  contract TEXT NOT NULL,
  creator TEXT NOT NULL,
  hash TEXT PRIMARY KEY NOT NULL,
  parsed BOOLEAN NOT NULL,
  verified BOOLEAN NOT NULL
);

CREATE TABLE evm_transactions_receipts (
  contract_address TEXT,
  cumulative_gas_used TEXT NOT NULL,
  effective_gas_price TEXT NOT NULL,
  gas_used TEXT NOT NULL,
  hash TEXT PRIMARY KEY, 
  status TEXT NOT NULL
);

CREATE TABLE evm_transactions_logs (
  address TEXT NOT NULL,
  topics TEXT[] NOT NULL, 
  data TEXT NOT NULL,
  hash TEXT NOT NULL,
  log_index BIGINT NOT NULL,
  removed BOOLEAN NOT NULL,
  PRIMARY KEY (hash, log_index)
);

INSERT INTO evm_methods (method, name) VALUES ('0x00000000', 'Transfer');

CREATE INDEX IF NOT EXISTS evm_blocks_by_chain
ON evm_blocks (chain);

CREATE INDEX IF NOT EXISTS evm_blocks_by_number
ON evm_blocks (number);

CREATE INDEX IF NOT EXISTS evm_transactions_by_block_number 
ON evm_transactions (block_number DESC);

CREATE INDEX IF NOT EXISTS evm_transactions_by_sender
ON evm_transactions (from_address);

CREATE INDEX IF NOT EXISTS evm_transactions_by_receiver
ON evm_transactions (to_address);

CREATE INDEX IF NOT EXISTS evm_transactions_by_chain
ON evm_transactions (chain);

CREATE INDEX IF NOT EXISTS evm_transactions_by_timestamp
ON evm_transactions (timestamp DESC);

CREATE INDEX IF NOT EXISTS evm_transactions_logs_by_hash 
ON evm_transactions_logs (hash);

CREATE INDEX IF NOT EXISTS evm_abis_by_contract
ON evm_abis (contract);