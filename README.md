# EVM Indexer

> Minimalistic EVM-compatible blockchain indexer written in rust.

This repository contains a program to parse all blocks and transactions into a PostgreSQL database. It also includes all the transaction receipts and logs for contract execution.

- [Database Structure](./doc/DATABASE.md)

## Demo

To see the EVM indexer in action go to [https://evm-indexer.kindynos.mx](https://evm-indexer.kindynos.mx)

The repository comes with an autodeployed Hasura Cloud instance to connect a GraphQL API directly to the indexer to fetch the data.

It is only enable if you use the docker-compose file.

The frontend app repository can be found here [https://github.com/eabz/evm-indexer-app](https://github.com/eabz/evm-indexer-app)

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

3. Copy the .env.example file to .env and add your environment variables.

```
DATABASE_URL -> URL for postgresql database.
PROVIDER_KEY -> API key for the RPC endpoint tested providers.
```

The available providers for the indexer are:

- [LlamaNodes](https://llamanodes.com)
- [Ankr](https://www.ankr.com/rpc)

4. Run the program

```
./target/release/evm-indexer
```

### Docker

The code has a builtin docker file that you can build through it, or you can use the constructed automatically image from

```
docker pull ghcr.io/eabz/evm-indexer:latest
```

For docker-compose, you can use

```
docker-compose up
```
