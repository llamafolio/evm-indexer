CREATE TABLE logs (
  hash VARCHAR PRIMARY KEY UNIQUE,
  address VARCHAR,
  topics text[],
  data VARCHAR,
  log_index BIGINT,
  transaction_log_index BIGINT,
  log_type VARCHAR
)