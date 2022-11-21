CREATE TABLE blocks (
  number BIGINT PRIMARY KEY,
  hash VARCHAR NOT NULL,
  difficulty  VARCHAR NOT NULL,
  total_difficulty  VARCHAR NOT NULL,
  miner VARCHAR NOT NULL,
  gas_limit VARCHAR NOT NULL,
  gas_used VARCHAR NOT NULL,
  txs BIGINT NOT NULL,
  timestamp BIGINT NOT NULL,
  size BIGINT NOT NULL,
  nonce VARCHAR NOT NULL,
  base_fee_per_gas VARCHAR NOT NULL
)