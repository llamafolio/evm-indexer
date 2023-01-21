CREATE TABLE chains_indexed_state (
  chain TEXT PRIMARY KEY NOT NULL,
  indexed_blocks_amount BIGINT NOT NULL
);