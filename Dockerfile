FROM clux/muslrust:stable AS chef

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

RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl --bin evm-indexer

FROM alpine AS runtime

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/evm-indexer /usr/local/bin/

CMD ["/usr/local/bin/evm-indexer"]