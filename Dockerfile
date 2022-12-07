FROM lukemathwalker/cargo-chef:latest AS chef

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release

FROM alpine:latest AS runtime

COPY --from=builder /target/release/evm-indexer /usr/local/bin/evm-indexer

ENTRYPOINT ["/usr/local/bin/evm-indexer"]
