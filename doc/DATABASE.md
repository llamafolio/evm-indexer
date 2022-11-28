### Database Structure

The database structure and information use the [diesel](https://crates.io/crates/diesel) crate.

Migrations for the database run every time you start the program.

There are ten tables created on the database.

- [Blocks (blocks)](#blocks-table)
- [Transactions (txs)](#transactions-table)
- [Transactions Receipts (txs_receipts)](#transactions-receipts-table)
- [Logs (logs)](#logs-table)
- [Contract Creation (contract_creation)](#contract-creation-table)
- [Contract Interactions (contract_interactions)](#contract-interactions-table)
- [Token Transfers (token_transfers)](#token-transfers-table)
- [State (state)](#state-table)
- [Tokens (tokens)](#tokens-table)
- [Excluded Tokens (excluded_tokens)](#exlucded-tokens-table)

#### Blocks Table

| Column             | PostgreSQL type | Rust type |
| ------------------ | --------------- | --------- |
| `number`           | `BIGINT`        | `i64 `    |
| `hash`             | `VARCHAR `      | `String ` |
| `difficulty`       | `VARCHAR `      | `String ` |
| `total_difficulty` | `VARCHAR `      | `String ` |
| `miner`            | `VARCHAR `      | `String ` |
| `gas_limit`        | `VARCHAR `      | `String ` |
| `gas_used`         | `VARCHAR `      | `String ` |
| `txs`              | `BIGINT `       | `i64 `    |
| `timestamp`        | `VARCHAR `      | `String ` |
| `size`             | `VARCHAR `      | `String ` |
| `nonce`            | `VARCHAR `      | `String ` |
| `base_fee_per_gas` | `VARCHAR `      | `String ` |
| `chain`            | `VARCHAR `      | `String ` |

#### Transactions Table

| Column                     | PostgreSQL type | Rust type |
| -------------------------- | --------------- | --------- |
| `hash `                    | `VARCHAR`       | `String ` |
| `block_number`             | `BIGINT `       | `i64 `    |
| `from_address`             | `VARCHAR `      | `String ` |
| `to_address`               | `VARCHAR `      | `String ` |
| `value`                    | `VARCHAR `      | `String ` |
| `gas_used`                 | `VARCHAR `      | `String ` |
| `gas_price`                | `VARCHAR `      | `String ` |
| `transaction_index`        | `BIGINT `       | `i64 `    |
| `transaction_type`         | `VARCHAR `      | `String ` |
| `max_fee_per_gas`          | `VARCHAR `      | `String ` |
| `max_priority_fee_per_gas` | `VARCHAR `      | `String ` |
| `input`                    | `VARCHAR `      | `String ` |
| `chain`                    | `VARCHAR `      | `String ` |

#### Transactions Receipts Table

| Column    | PostgreSQL type | Rust type |
| --------- | --------------- | --------- |
| `hash `   | `VARCHAR`       | `String ` |
| `success` | `BOOLEAN `      | `bool `   |
| `chain`   | `VARCHAR `      | `String ` |

#### Logs Table

| Column                  | PostgreSQL type | Rust type      |
| ----------------------- | --------------- | -------------- |
| `hash `                 | `VARCHAR`       | `String `      |
| `address`               | `VARCHAR `      | `String `      |
| `topics`                | `text[] `       | `Vec<String> ` |
| `data`                  | `VARCHAR `      | `String `      |
| `log_index`             | `BIGINT `       | `i64 `         |
| `transaction_log_index` | `BIGINT `       | `i64 `         |
| `log_type`              | `VARCHAR `      | `String `      |
| `chain`                 | `VARCHAR `      | `String `      |

#### Contract Creation Table

| Column     | PostgreSQL type | Rust type |
| ---------- | --------------- | --------- |
| `hash `    | `VARCHAR`       | `String ` |
| `block`    | `BIGINT`        | `i64 `    |
| `contract` | `VARCHAR`       | `String`  |
| `chain`    | `VARCHAR `      | `String ` |

#### Contract Interactions Table

| Column     | PostgreSQL type | Rust type |
| ---------- | --------------- | --------- |
| `hash `    | `VARCHAR`       | `String ` |
| `address`  | `VARCHAR`       | `String ` |
| `block`    | `BIGINT`        | `i64 `    |
| `contract` | `VARCHAR`       | `String`  |
| `chain`    | `VARCHAR `      | `String ` |

#### Token Transfers Table

| Column            | PostgreSQL type | Rust type |
| ----------------- | --------------- | --------- |
| `hash_with_index` | `VARCHAR`       | `String ` |
| `hash`            | `VARCHAR`       | `String ` |
| `block`           | `BIGINT`        | `i64 `    |
| `log_index`       | `BIGINT`        | `i64 `    |
| `from_address`    | `VARCHAR`       | `String ` |
| `to_address`      | `VARCHAR`       | `String ` |
| `value`           | `VARCHAR`       | `String ` |
| `token`           | `VARCHAR`       | `String`  |
| `chain`           | `VARCHAR `      | `String ` |

#### State Table

| Column   | PostgreSQL type | Rust type |
| -------- | --------------- | --------- |
| `chain`  | `VARCHAR`       | `String ` |
| `blocks` | `BIGINT`        | `String ` |

#### Tokens Table

| Column               | PostgreSQL type | Rust type |
| -------------------- | --------------- | --------- |
| `address_with_chain` | `VARCHAR`       | `String ` |
| `address`            | `VARCHAR`       | `String ` |
| `chain`              | `VARCHAR`       | `String ` |
| `name`               | `VARCHAR`       | `String ` |
| `decimals`           | `BIGINT`        | `i64 `    |
| `symbol`             | `VARCHAR`       | `String ` |

#### Excluded Tokens Table

| Column               | PostgreSQL type | Rust type |
| -------------------- | --------------- | --------- |
| `address_with_chain` | `VARCHAR`       | `String ` |
| `address`            | `VARCHAR`       | `String ` |
| `chain`              | `VARCHAR`       | `String ` |
