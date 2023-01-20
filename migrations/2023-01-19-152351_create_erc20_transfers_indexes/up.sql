CREATE INDEX IF NOT EXISTS evm_erc20_transfers_by_hash 
ON evm_erc20_transfers (hash);

CREATE INDEX IF NOT EXISTS evm_erc20_transfers_by_sender
ON evm_erc20_transfers (from_address);

CREATE INDEX IF NOT EXISTS evm_erc20_transfers_by_receiver
ON evm_erc20_transfers (to_address);
