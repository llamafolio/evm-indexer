DROP TABLE evm_blocks;
DROP TABLE evm_transactions;
DROP TABLE evm_transactions_receipts;
DROP TABLE evm_transactions_logs;
DROP TABLE evm_methods;
DROP TABLE evm_contracts;
DROP TABLE evm_abis;

DROP INDEX evm_blocks_by_chain;
DROP INDEX evm_blocks_by_number;
DROP INDEX evm_transactions_by_block_number;
DROP INDEX evm_transactions_by_sender;
DROP INDEX evm_transactions_by_receiver;
DROP INDEX evm_transactions_by_chain;
DROP INDEX evm_transactions_by_timestamp;
DROP INDEX evm_transactions_logs_by_hash;
DROP INDEX evm_abis_by_contract;