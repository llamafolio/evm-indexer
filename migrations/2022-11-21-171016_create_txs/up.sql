CREATE TABLE txs (
  hash VARCHAR PRIMARY KEY UNIQUE,
  block_number BIGINT NOT NULL,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  value VARCHAR NOT NULL,
  gas_used VARCHAR NOT NULL,
  gas_price VARCHAR NOT NULL,
  transaction_index BIGINT NOT NULL,
  transaction_type BIGINT,
  max_fee_per_gas VARCHAR,
  max_priority_fee_per_gas VARCHAR,
  input VARCHAR NOT NULL
)