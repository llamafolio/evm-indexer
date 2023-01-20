ALTER TABLE evm_erc20_transfers DROP CONSTRAINT evm_erc20_transfers_pkey;
ALTER TABLE evm_erc20_transfers ADD CONSTRAINT erc20_transfers_table_pk PRIMARY KEY (hash, log_index);