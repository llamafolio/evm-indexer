<h1 align="center">
<strong>EVM Blockchain Indexer</strong>
</h1>
<p align="center">
<strong>A scalable SQL indexer for EVM compatible blockchains</strong>
</p>

The indexer is ready and used in production. If you want to use it or contribute and need help you can [send me a DM on my personal twitter.](https://twitter.com/eaberrueta)

If you want to see it in action, we have a small API to showcase at https://indexer.kindynos.mx

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [CockroachDB](https://www.cockroachlabs.com/) (or any other PostgreSQL db)
- [Redis](https://redis.io/) (used to store the indexed blocks state)
- [Docker](https://www.docker.com/) (Optional for the Hasura Cloud deploy)

## Available Chains

This indexer is chain agnostic. It should work with any chain that follows the ETH RPC API. But some chains have some minor modifications that could result in them not being able to sync.

The following chains have been tested and indexed successfully:

- Arbitrum One.
- Arbitrum Nova.
- Avalanche.
- BitTorrent Chain.
- BNB Chain.
- Celo.
- Ethereum.
- Fantom.
- Gnosis Chain.
- Moonbeam.
- Optimism.
- Polygon.

## Install

You can try the indexer locally or through Docker.

### Local

1. Clone the repository

```
git clone https://github.com/llamafolio/evm-indexer && cd evm-indexer
```

2. Build the program

```
cargo build --release
```

3. Copy the `.env.example` file to `.env` and add your environment variables.

4. Run the program

`TODO: programs and flags.`

### Docker

1. Clone the repository

```
git clone https://github.com/llamafolio/evm-indexer && cd evm-indexer
```

2. Build the image and tag it as `indexer`

```
docker build . -t indexer
```

3. Copy the `.env.example` file to `.env` and add your environment variables.

4. Run the image

`TODO: programs and flags.`

## Deploy a GraphQL API with Hasura.

The repository contains [Hasura Cloud](https://hasura.io/cloud/) docker compose file to deploy it together with the indexer.

1. Copy the `.env.example` file to `.env` and add your environment variables for the Hasura Cloud.

2. Run the docker compose file

```
docker-compose up -d
```
