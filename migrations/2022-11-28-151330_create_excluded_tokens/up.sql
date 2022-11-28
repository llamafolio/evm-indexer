CREATE TABLE excluded_tokens (
  address_with_chain VARCHAR PRIMARY KEY UNIQUE NOT NULL,
  address VARCHAR NOT NULL,
  chain VARCHAR NOT NULL
)