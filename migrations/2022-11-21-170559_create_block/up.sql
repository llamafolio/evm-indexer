CREATE TABLE blocks (
  number BIGINT NOT NULL,
  hash VARCHAR NOT NULL PRIMARY KEY,
  difficulty  VARCHAR NOT NULL,
  total_difficulty  VARCHAR NOT NULL,
  miner VARCHAR NOT NULL,
  gas_limit VARCHAR NOT NULL,
  gas_used VARCHAR NOT NULL,
  txs BIGINT NOT NULL,
  timestamp VARCHAR NOT NULL,
  size VARCHAR NOT NULL,
  nonce VARCHAR NOT NULL,
  base_fee_per_gas VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
)