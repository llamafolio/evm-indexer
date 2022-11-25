CREATE TABLE token_transfers (
  hash_with_index VARCHAR PRIMARY KEY UNIQUE NOT NULL, 
  block BIGINT NOT NULL,
  token VARCHAR NOT NULL,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  value VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
)