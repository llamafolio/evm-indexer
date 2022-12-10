FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

USER root

RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner

WORKDIR /app

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --bin evm-indexer

FROM debian:stable AS runtime

RUN apt update && apt install -y libpq5

COPY --from=builder /app/target/release/evm-indexer /usr/local/bin/

CMD ["/usr/local/bin/evm-indexer"]