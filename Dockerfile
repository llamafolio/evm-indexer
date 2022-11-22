FROM rust:latest as builder

WORKDIR /

ADD . .

RUN cargo build --workspace --release

FROM rust:latest

COPY --from=builder /evm-indexer/target/production/evm-indexer /usr/local/bin/evm-indexer
