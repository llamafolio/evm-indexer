CREATE TABLE evm_erc20_transfers (
  hash VARCHAR NOT NULL, 
  log_index BIGINT NOT NULL,
  block BIGINT NOT NULL,
  token VARCHAR NOT NULL,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  value VARCHAR NOT NULL,
  chain VARCHAR NOT NULL,
  PRIMARY KEY (hash, log_index)
);