CREATE TABLE nft_transfers (
  hash_with_index VARCHAR PRIMARY KEY UNIQUE NOT NULL, 
  hash VARCHAR NOT NULL, 
  log_index BIGINT NOT NULL,
  log_type VARCHAR NOT NULL,
  block BIGINT NOT NULL,
  token VARCHAR NOT NULL,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  token_id VARCHAR NOT NULL,
  amount VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);

CREATE TABLE nfts (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL,
  nft_type VARCHAR NOT NULL,
  name VARCHAR NOT NULL,
  symbol VARCHAR NOT NULL
);

CREATE TABLE excluded_nfts (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
);