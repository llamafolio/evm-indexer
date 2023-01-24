CREATE TABLE evm_erc20_balances (
  address TEXT NOT NULL,
  chain TEXT NOT NULL, 
  token TEXT NOT NULL, 
  sent TEXT NOT NULL, 
  received TEXT NOT NULL, 
  PRIMARY KEY (address, token, chain)
);

ALTER TABLE evm_erc20_transfers RENAME COLUMN erc20_tokens_parced TO erc20_tokens_parsed;

ALTER TABLE evm_erc20_transfers ADD COLUMN erc20_balances_parsed BOOL;

CREATE INDEX IF NOT EXISTS evm_erc20_balances_by_token
ON evm_erc20_balances (token);

CREATE INDEX IF NOT EXISTS evm_erc20_balances_by_address
ON evm_erc20_balances (address);
