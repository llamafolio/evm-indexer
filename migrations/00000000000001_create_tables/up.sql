CREATE TABLE abis (
  abi TEXT, 
  chain TEXT NOT NULL, 
  contract TEXT NOT NULL, 
  verified BOOLEAN NOT NULL, 
  CONSTRAINT abis_pkey PRIMARY KEY (contract, chain)
);

CREATE INDEX IF NOT EXISTS abis_by_contract ON abis (contract);

CREATE TABLE blocks (
  base_fee_per_gas TEXT NOT NULL, 
  block_hash TEXT NOT NULL, 
  chain TEXT NOT NULL, 
  difficulty TEXT NOT NULL, 
  extra_data TEXT NOT NULL, 
  gas_limit TEXT NOT NULL, 
  gas_used TEXT NOT NULL, 
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
  uncles TEXT[] NOT NULL, 
  CONSTRAINT blocks_pkey PRIMARY KEY (block_hash)
);

CREATE INDEX IF NOT EXISTS blocks_by_chain ON blocks (chain);

CREATE INDEX IF NOT EXISTS blocks_by_number ON blocks (number);

CREATE TABLE transactions (
  block_hash TEXT NOT NULL,
  block_number BIGINT NOT NULL, 
  chain TEXT NOT NULL,
  from_address TEXT NOT NULL, 
  gas TEXT NOT NULL, 
  gas_price TEXT NOT NULL, 
  hash TEXT NOT NULL,  
  input TEXT NOT NULL, 
  max_fee_per_gas TEXT, 
  max_priority_fee_per_gas TEXT, 
  method TEXT NOT NULL,
  nonce TEXT NOT NULL, 
  timestamp TEXT NOT NULL, 
  to_address TEXT NOT NULL, 
  transaction_index BIGINT NOT NULL, 
  transaction_type BIGINT, 
  value TEXT NOT NULL,
  CONSTRAINT transactions_pkey PRIMARY KEY (hash)
);

CREATE INDEX IF NOT EXISTS transactions_by_block_number ON transactions (block_number DESC);

CREATE INDEX IF NOT EXISTS transactions_by_sender ON transactions (from_address) STORING (to_address);  

CREATE INDEX IF NOT EXISTS transactions_by_receiver ON transactions (to_address) STORING (from_address); 

CREATE INDEX IF NOT EXISTS transactions_by_chain ON transactions (chain);

CREATE INDEX IF NOT EXISTS transactions_by_timestamp ON transactions (timestamp DESC);

CREATE TABLE methods (
  method TEXT NOT NULL, 
  name TEXT NOT NULL, 
  CONSTRAINT methods_pkey PRIMARY KEY (method)
);

INSERT INTO methods (method, name) 
VALUES 
  ('0x00000000', 'Transfer');

CREATE TABLE chains_indexed_state (
  chain TEXT NOT NULL, 
  indexed_blocks_amount BIGINT NOT NULL, 
  CONSTRAINT chains_indexed_state_pkey PRIMARY KEY (chain)
);

CREATE TABLE contracts_adapters (
  adapter_id TEXT NOT NULL, 
  address TEXT NOT NULL, 
  chain TEXT NOT NULL, 
  CONSTRAINT contracts_adapters_pkey PRIMARY KEY (address, chain)
);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_address ON contracts_adapters (address);

CREATE INDEX IF NOT EXISTS contracts_adapters_by_adapter_id ON contracts_adapters (adapter_id);

CREATE TABLE erc20_transfers (
  chain TEXT NOT NULL, 
  erc20_balances_parsed BOOLEAN NOT NULL, 
  erc20_tokens_parsed BOOLEAN NOT NULL, 
  from_address TEXT NOT NULL, 
  hash TEXT NOT NULL, 
  log_index BIGINT NOT NULL, 
  to_address TEXT NOT NULL, 
  token TEXT NOT NULL, 
  value TEXT NOT NULL, 
  PRIMARY KEY (hash, log_index)
);

CREATE INDEX IF NOT EXISTS erc20_transfers_by_token ON erc20_transfers (token);

CREATE INDEX IF NOT EXISTS erc20_transfers_by_hash ON erc20_transfers (hash);

CREATE INDEX IF NOT EXISTS erc20_transfers_by_sender ON erc20_transfers (from_address) STORING (to_address);

CREATE INDEX IF NOT EXISTS erc20_transfers_by_receiver ON erc20_transfers (to_address) STORING (from_address);  

CREATE INDEX IF NOT EXISTS erc20_transfers_by_erc20_tokens_parsed ON erc20_transfers (erc20_tokens_parsed) STORING (chain, erc20_balances_parsed, from_address, to_address, token, value);

CREATE INDEX IF NOT EXISTS erc20_transfers_by_erc20_balances_parsed ON erc20_transfers (erc20_balances_parsed) STORING (chain, erc20_tokens_parsed, from_address, to_address, token, value);

CREATE TABLE erc20_tokens (
  address TEXT NOT NULL, 
  chain TEXT NOT NULL, 
  decimals BIGINT, 
  name TEXT, 
  symbol TEXT, 
  PRIMARY KEY (address, chain)
);

CREATE INDEX IF NOT EXISTS erc20_tokens_by_address ON erc20_tokens (address);

CREATE INDEX IF NOT EXISTS erc20_tokens_by_chain ON erc20_tokens (chain);

CREATE TABLE erc20_balances (
  address TEXT NOT NULL, 
  balance DOUBLE PRECISION NOT NULL, 
  chain TEXT NOT NULL, 
  token TEXT NOT NULL, 
  PRIMARY KEY (address, token, chain)
);

CREATE INDEX IF NOT EXISTS erc20_balances_by_token ON erc20_balances (token, chain);

CREATE INDEX IF NOT EXISTS erc20_balances_by_address ON erc20_balances (address, chain);

CREATE INDEX IF NOT EXISTS erc20_balances_by_balance ON erc20_balances (balance DESC);

CREATE TABLE receipts (
  contract_address TEXT,
  cumulative_gas_used TEXT NOT NULL,
  effective_gas_price TEXT NOT NULL,
  gas_used TEXT NOT NULL,
  hash TEXT NOT NULL, 
  status TEXT NOT NULL,
  CONSTRAINT receipts_pkey PRIMARY KEY (hash)
);

CREATE TABLE contracts (
  block BIGINT NOT NULL,
  chain TEXT NOT NULL, 
  contract TEXT NOT NULL,
  creator TEXT NOT NULL,
  hash TEXT NOT NULL,
  parsed BOOLEAN NOT NULL,
  verified BOOLEAN NOT NULL,
  CONSTRAINT contracts_pkey PRIMARY KEY (contract, chain)
);

CREATE INDEX IF NOT EXISTS contracts_by_contract ON contracts (contract);

CREATE INDEX IF NOT EXISTS contracts_by_hash ON contracts (hash);

CREATE TABLE logs (
  address TEXT NOT NULL,
  chain TEXT NOT NULL,
  data TEXT NOT NULL,
  erc20_transfers_parsed BOOLEAN NOT NULL,
  hash TEXT NOT NULL,
  log_index BIGINT NOT NULL,
  removed BOOLEAN NOT NULL,
  topics TEXT[] NOT NULL, 
  CONSTRAINT logs_pkey PRIMARY KEY (hash, log_index)
);

CREATE INDEX IF NOT EXISTS logs_by_address ON logs (address, chain);

CREATE INDEX IF NOT EXISTS logs_by_hash ON logs (hash);

CREATE INDEX IF NOT EXISTS logs_by_erc20_transfers_parsed ON logs (erc20_transfers_parsed) STORING (address, chain, data, removed, topics);

