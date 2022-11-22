FROM rust:latest as builder

WORKDIR /

COPY Cargo.toml Cargo.lock /app/

WORKDIR /app/
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY ./src /app/src
COPY ./bin /app/bin

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  touch /app/game/src/lib.rs /app/api/src/main.rs
  cargo build --release
EOF

FROM rust:latest

COPY --from=builder /evm-indexer/target/release/evm-indexer /usr/local/bin/evm-indexer
