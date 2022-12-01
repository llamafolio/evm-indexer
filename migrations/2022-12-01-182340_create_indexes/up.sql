CREATE INDEX IF NOT EXISTS token_transfers_by_hash 
ON token_transfers USING HASH (hash);

CREATE INDEX IF NOT EXISTS token_transfers_by_sender
ON token_transfers USING HASH (from_address);

CREATE INDEX IF NOT EXISTS token_transfers_by_receiver
ON token_transfers USING HASH (to_address);

CREATE INDEX IF NOT EXISTS tokens_by_chain 
ON tokens USING HASH (chain);

CREATE INDEX IF NOT EXISTS tokens_by_address 
ON tokens USING HASH (address);

CREATE INDEX IF NOT EXISTS txs_by_block_number 
ON txs USING BTREE (block_number DESC);

CREATE INDEX IF NOT EXISTS txs_by_sender 
ON txs USING HASH (from_address);

CREATE INDEX IF NOT EXISTS txs_by_receiver
ON txs USING HASH (to_address);
