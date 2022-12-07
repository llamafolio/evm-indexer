FROM lukemathwalker/cargo-chef:latest AS chef

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --bin evm-indexer


FROM alpine:latest AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/evm-indexer /usr/local/bin

ENTRYPOINT ["/usr/local/bin/evm-indexer"]
