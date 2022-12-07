FROM rust:latest as builder

WORKDIR /

RUN cargo new --lib /app/
COPY Cargo.toml Cargo.lock /app/


WORKDIR /app/
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY ./src /app/src/
COPY ./migrations /app/migrations/

RUN touch /app/src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release


FROM rust:latest

COPY --from=builder /app/target/release/evm-indexer /usr/local/bin/evm-indexer