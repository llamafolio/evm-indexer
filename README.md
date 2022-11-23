# EVM Indexer

> Minimalistic EVM-compatible blockchain indexer written in rust.

This repository contains a program to parse all blocks and transactions into a PostgreSQL database. It also includes all the transaction receipts and logs for contract execution.

## Disclaimer

This program is highly experimental and not meant to be used for production.

## Install

You can try the indexer locally or through Docker.

### Local

To use the program locally, make sure you have [rust](https://www.rust-lang.org/tools/install) installed in your environment.

1. Clone the repository

```bash
git clone https://github.com/eabz/evm-indexer && cd evm-indexer
```

2. Build the program

```bash
cargo build --release
```

3. Copy the .env.example file to .env and add your environment variables.

```
DATABASE_URL -> URL for postgresql database.
RPC_WS_URL -> Websocket URL for the EVM-blockchain RPC endpoint (for new blocks).
RPC_HTTPS_URL -> HTTP URL for the EVM-blockchain RPC endpoint (to fetch past blocks).
```

4. Run the program

```bash
./target/release/evm-indexer
```

### Docker

The code has a builtin docker file that you can build through it, or you can use the constructed automatically image from

```bash
docker pull ghcr.io/eabz/evm-indexer:latest
```

For docker-compose, you can use

```bash
docker-compose up
```
