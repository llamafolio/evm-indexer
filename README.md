# EVM Indexer

> Minimalistic EVM-compatible blockchain indexer written in rust.

This repository contains a program to parse all blocks and transactions into a PostgreSQL database. It also includes all the transaction receipts and logs for contract execution.

- [Database Structure](./doc/DATABASE.md)

## Disclaimer

This program is highly experimental and not meant to be used for production.

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

- [Ankr](https://www.ankr.com/rpc/)

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
