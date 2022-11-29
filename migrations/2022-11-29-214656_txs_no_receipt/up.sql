CREATE TABLE txs_no_receipt (
  hash VARCHAR PRIMARY KEY UNIQUE,
  chain VARCHAR NOT NULL,
  block_number BIGINT NOT NULL
)