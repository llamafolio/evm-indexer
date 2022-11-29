# EVM Indexer

> Minimalistic EVM-compatible blockchain indexer written in rust.

This repository contains a program to index helpful information from any EVM-compatible chain into a PostgreSQL database.

It is ready for developing purposes. If you want more information about it, you can [send me a DM on Twitter](https://twitter.com/eaberrueta)

## Demo

To see the EVM indexer in action, go to [https://evm-indexer.kindynos.mx](https://evm-indexer.kindynos.mx)

The frontend app repository can be found here [https://github.com/eabz/evm-indexer-app](https://github.com/eabz/evm-indexer-app)

## Chains

Currently, the indexer can index the following chains:

- Ethereum (mainnet)
- Polygon
- Avalanche
- Fantom
- Gnosis Chain
- Optimism

## Database Information

The indexer creates tables for:

1. Blocks
2. Transactions
3. Transaction Receipts
4. Transaction Logs
5. Contract Creations
6. Contract Interactions
7. Token Transfers
8. Tokens Details

The information structure is explained in the [database structure document](./doc/DATABASE.md).

## Providers

The indexer connects automatically to three different providers through the RPC.

The available providers are:

- [LlamaNodes](https://llamanodes.com)
- [Ankr](https://www.ankr.com/rpc)
- [Pokt](https://www.pokt.network/)

The indexer automatically selects the providers added.

## Environment Variables

The indexer requires the following environment variables.

| Variable                 | Purpose                              | Required Local | Required Docker |
| ------------------------ | ------------------------------------ | -------------- | --------------- |
| `DATABASE_URL`           | Url of the PostgreSQL database       | `true`         | `false `        |
| `ANKR_PROVIDER_ID`       | Ankr RPC nodes provider ID           | `false `       | `false `        |
| `LLAMANODES_PROVIDER_ID` | LlamaNodes RPC nodes provider ID     | `false `       | `false `        |
| `POKT_PROVIDER_ID`       | Pokt RPC provider ID                 | `false `       | `false `        |
| `HASURA_ADMIN_PASSWORD`  | Hasura console and GraphQL API token | `false `       | `true `         |

## Install

You can try the indexer locally or through Docker.

### Local

To use the program locally, make sure you have [rust](https://www.rust-lang.org/tools/install) installed in your environment.

1. Clone the repository

```
git clone https://github.com/eabz/evm-indexer && cd evm-indexer
```

2. Build the program

```
cargo build --release
```

3. Copy the `.env.example` file to `.env` and add your environment variables.

4. Run the program

```
./target/release/evm-indexer
```

### Docker

You can use the official docker image.

```
docker pull ghcr.io/eabz/evm-indexer:latest
```

You can use our docker-compose script to start a full indexer with a database, all chains enabled, and a Hasura Cloud GraphQL API.

```
docker-compose up
```

### Contribute

We appreciate your contributions. PR are accepted and open.

Some ideas for contributions are:

1. Add more chains
2. Increment providers to sync simultaneously.
3. Speed up the information deserialization/storing.
