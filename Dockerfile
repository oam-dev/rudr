FROM rust:1.37 AS builder
WORKDIR /usr/src/scylla

COPY Cargo.toml .
COPY Cargo.lock .

# Create dummy files to build the dependencies, cargo won't build without src/main.rs and src/lib.rs
# then remove Scylla fingerprint for following rebuild
RUN mkdir -p ./src/ && \
    echo 'fn main() {}' > ./src/main.rs && \
    echo '' > ./src/lib.rs
RUN cargo build --release && \
    rm -rf ./target/release/.fingerprint/scylla-*

# Build real binaries now
COPY ./src ./src
RUN cargo build --release

FROM debian:stretch-slim
WORKDIR /usr/app
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/scylla/target/release/scylla .
ENV RUST_LOG scylla=info
CMD ["./scylla"]