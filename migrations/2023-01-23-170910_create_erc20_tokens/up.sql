CREATE TABLE evm_erc20_tokens (
  address TEXT NOT NULL,
  chain TEXT NOT NULL, 
  name TEXT,
  decimals BIGINT,
  symbol TEXT,
  PRIMARY KEY (address, chain)
);

CREATE INDEX IF NOT EXISTS evm_erc20_tokens_by_address
ON evm_erc20_tokens (address);

CREATE INDEX IF NOT EXISTS evm_erc20_tokens_by_chain
ON evm_erc20_tokens (chain);

ALTER TABLE evm_erc20_transfers ADD COLUMN erc20_tokens_parced BOOL;