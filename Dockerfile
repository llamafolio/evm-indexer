FROM rust:latest as builder

WORKDIR /

RUN cargo new --lib /app/
COPY Cargo.toml Cargo.lock /app/

WORKDIR /app/
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY ./src /app/src/
COPY ./bin /app/bin/

RUN ls 
RUN ls /app/src/
RUN ls /app/bin/


RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  touch /app/src/lib.rs /app/bin/evm-indexer.rs
  cargo build --release
EOF

RUN ls ./app/target


FROM rust:latest

COPY --from=builder /app/target/release/evm-indexer /usr/local/bin/evm-indexer
