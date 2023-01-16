# EVM Indexer

> EVM-compatible blockchain indexer written in rust.

This repository contains a program to index helpful information from any EVM-compatible chain into a PostgreSQL database.

It is ready for developing purposes. If you want more information about it, you can [send me a DM on Twitter](https://twitter.com/eaberrueta)

## Demo

To see the EVM indexer in action, go to [https://dashboard.kindynos.mx](https://dashboard.kindynos.mx)

The frontend app repository can be found here [https://github.com/eabz/evm-indexer-app](https://github.com/eabz/evm-indexer-app)

## Chains

Currently, the indexer has been tested indexing the following chains:

- Ethereum
- Polygon
- Avalanche
- Fantom
- Gnosis Chain
- Optimism
- BNB Chain
- Dogechain
- Arbitrum

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
9. Contract ABIs (filled only if the abi source token is provided)

The information structure is explained in the [database structure documentation](./doc/DATABASE.md).

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

1. Clone the repository

```
git clone https://github.com/eabz/evm-indexer && cd evm-indexer
```

2. Build the image and tag it as `indexer`

```
docker build . -t indexer
```

3. Run the image

```
docker run --env-file ./.env indexer -d
```

## Contribute

We appreciate your contributions. PR are accepted and open.

Some ideas for contributions are:

1. Add more chains
2. Speed up the information deserialization/storing.
