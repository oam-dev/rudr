FROM rust:1.36 AS builder
WORKDIR /usr/src/scylla
COPY ./src ./src
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release
RUN ls -lah ./target/release

FROM debian:stretch-slim
WORKDIR /usr/app
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/scylla/target/release/scylla .
ENV RUST_LOG scylla=info
CMD ["./scylla"]