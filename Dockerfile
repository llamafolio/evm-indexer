FROM rust:latest as builder

WORKDIR /

RUN cargo new --lib /app/
COPY Cargo.toml Cargo.lock /app/


WORKDIR /app/
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY ./src /app/src/
COPY ./migrations /app/migrations/

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  touch /app/src/main.rs
  cargo build --release
EOF


FROM rust:latest

COPY --from=builder /app/target/release/evm-indexer /usr/local/bin/evm-indexer