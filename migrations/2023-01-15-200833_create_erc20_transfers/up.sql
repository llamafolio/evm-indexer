CREATE TABLE evm_erc20_transfers (
  hash TEXT NOT NULL, 
  log_index BIGINT NOT NULL,
  token TEXT NOT NULL,
  from_address TEXT NOT NULL,
  to_address TEXT NOT NULL,
  value TEXT NOT NULL,
  CONSTRAINT erc20_transfers_table_pk PRIMARY KEY (hash, log_index)
);