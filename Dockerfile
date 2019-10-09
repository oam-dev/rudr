ARG BUILDER_IMAGE=rust:1.37
ARG BASE_IMAGE=debian:stretch-slim

FROM ${BUILDER_IMAGE} AS builder
WORKDIR /usr/src/rudr

COPY Cargo.toml .
COPY Cargo.lock .

# Create dummy files to build the dependencies, cargo won't build without src/main.rs and src/lib.rs
# then remove Rudr fingerprint for following rebuild
RUN mkdir -p ./src/ && \
    echo 'fn main() {}' > ./src/main.rs && \
    echo '' > ./src/lib.rs
RUN cargo build --release && \
    rm -rf ./target/release/.fingerprint/rudr-*

# Build real binaries now
COPY ./src ./src
RUN cargo build --release

FROM ${BASE_IMAGE}
WORKDIR /usr/app
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rudr/target/release/rudr .
ENV RUST_LOG rudr=info
CMD ["./rudr"]