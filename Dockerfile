FROM rust:latest as builder

WORKDIR /

RUN cargo new --lib /app/
COPY Cargo.toml Cargo.lock /app/

# Remove the last 3 lines of the Cargo.toml that includes the bin to build the evm-indexer
RUN cat /app/Cargo.toml | head -n -3 > /app/Cargo.toml

WORKDIR /app/
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY ./src /app/src/
COPY ./bin /app/bin/
COPY ./migrations /app/migrations/

# Copy the cargo file again to build the bin
COPY Cargo.toml Cargo.lock /app/

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  touch /app/src/lib.rs /app/bin/evm-indexer.rs
  cargo build --release
EOF


FROM rust:latest

COPY --from=builder /app/target/release/evm-indexer /usr/local/bin/evm-indexer