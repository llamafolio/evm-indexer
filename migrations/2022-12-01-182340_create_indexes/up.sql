CREATE INDEX IF NOT EXISTS token_transfers_by_hash 
ON token_transfers(hash);

CREATE INDEX IF NOT EXISTS tokens_by_chain 
ON tokens(chain);

CREATE INDEX IF NOT EXISTS tokens_by_address 
ON tokens(address);

CREATE INDEX IF NOT EXISTS txs_by_block_number 
ON txs(block_number DESC);

CREATE INDEX IF NOT EXISTS txs_by_sender 
ON txs(from_address);

CREATE INDEX IF NOT EXISTS txs_by_receiver
ON txs(to_address);

