CREATE TABLE nft_transfers (
  chain TEXT NOT NULL, 
  nft_balances_parsed BOOLEAN NOT NULL, 
  nft_tokens_parsed BOOLEAN NOT NULL, 
  from_address TEXT NOT NULL, 
  to_address TEXT NOT NULL, 
  token TEXT NOT NULL, 
  token_id DECIMAL NOT NULL, 
  value DECIMAL NOT NULL, 
  hash TEXT NOT NULL, 
  log_index BIGINT NOT NULL, 
  transfer_index BIGINT NOT NULL, 
  transfer_type TEXT NOT NULL,
  PRIMARY KEY (hash, log_index, transfer_index)
);

CREATE INDEX IF NOT EXISTS nft_transfers_by_token ON nft_transfers (token, token_id, chain);

CREATE INDEX IF NOT EXISTS nft_transfers_by_hash ON nft_transfers (hash);

CREATE INDEX IF NOT EXISTS nft_transfers_by_sender ON nft_transfers (from_address) STORING (to_address);

CREATE INDEX IF NOT EXISTS nft_transfers_by_receiver ON nft_transfers (to_address) STORING (from_address);  

CREATE INDEX IF NOT EXISTS nft_transfers_by_nft_tokens_parsed ON nft_transfers (nft_tokens_parsed) STORING (chain, nft_balances_parsed, from_address, to_address, token, token_id, value);

CREATE INDEX IF NOT EXISTS nft_transfers_by_nft_balances_parsed ON nft_transfers (nft_balances_parsed) STORING (chain, nft_tokens_parsed, from_address, to_address, token, token_id, value);

CREATE TABLE nft_tokens (
  address TEXT NOT NULL, 
  chain TEXT NOT NULL, 
  nft_type TEXT NOT NULL,
  name TEXT,
  symbol TEXT,
  contract_uri TEXT,
  PRIMARY KEY (address, chain)
);

CREATE INDEX IF NOT EXISTS nft_tokens_by_address ON nft_tokens (address);

CREATE INDEX IF NOT EXISTS nft_tokens_by_chain ON nft_tokens (chain);

CREATE TABLE nft_balances (
  address TEXT NOT NULL, 
  chain TEXT NOT NULL, 
  token TEXT NOT NULL, 
  token_id DECIMAL NOT NULL,
  balance DECIMAL NOT NULL,
  PRIMARY KEY (address, token, chain)
);

CREATE INDEX IF NOT EXISTS nft_balances_by_token ON nft_balances (token, token_id, chain);

CREATE INDEX IF NOT EXISTS nft_balances_by_address ON nft_balances (address, chain);

CREATE INDEX IF NOT EXISTS nft_balances_by_balance ON nft_balances (balance DESC);

ALTER TABLE logs ADD COLUMN nft_transfers_parsed BOOLEAN NOT NULL;

CREATE INDEX IF NOT EXISTS logs_by_nft_transfers_parsed ON logs (nft_transfers_parsed) STORING (address, chain, data, removed, topics);

CREATE TABLE nft_token_uris (
  token TEXT NOT NULL, 
  token_id DECIMAL NOT NULL,
  chain TEXT NOT NULL,
  token_uri TEXT,
  is_parsed BOOLEAN NOT NULL,
  PRIMARY KEY (token, token_id, chain)
)
