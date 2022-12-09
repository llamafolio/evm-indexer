# Blocks

## Array

txs_hash -> txs.(chain, block_number ) -> blocks.(chain,number)

# Token Transfers

## Object

token_details -> token_transfers.(chain,token) -> tokens.(chain,address)

# Contract Interactions

## Object

method_name -> contract_interaction.method_id -> method_ids.name

# Transactions

## Object

contract_created -> txs.hash -> contract_creations.hash
contract_interaction -> txs.hash -> contract_interactions.hash
receip -> txs.hash -> txs_receipts.hash

## Array

logs -> logs.hash -> txs.hash
token_transfers -> token_transfers.hash -> txs.hash
