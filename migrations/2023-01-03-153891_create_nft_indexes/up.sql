CREATE INDEX IF NOT EXISTS nft_transfers_by_hash 
ON nft_transfers (hash);

CREATE INDEX IF NOT EXISTS nft_transfers_by_sender
ON nft_transfers (from_address);

CREATE INDEX IF NOT EXISTS nft_transfers_by_receiver
ON nft_transfers (to_address);

CREATE INDEX IF NOT EXISTS nft_by_chain 
ON nfts (chain);

CREATE INDEX IF NOT EXISTS nft_by_address 
ON nfts (address);