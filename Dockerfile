ARG BUILDER_IMAGE=rust:1.38
ARG BASE_IMAGE=debian:buster-slim
ARG PACKAGE_NAME=rudr
FROM ${BUILDER_IMAGE} AS builder
WORKDIR /usr/src/rudr
RUN mkdir healthscope

COPY Cargo.toml .
COPY Cargo.lock .
COPY healthscope/Cargo.toml ./healthscope/

# Create dummy files to build the dependencies, cargo won't build without src/main.rs and src/lib.rs
# then remove Rudr fingerprint for following rebuild
RUN mkdir -p ./src/ && \
    echo 'fn main() {}' > ./src/main.rs && \
    echo '' > ./src/lib.rs && \
    mkdir -p ./healthscope/src/ && \
    echo 'fn main() {}' > ./healthscope/src/main.rs && \
    echo '' > ./healthscope/src/lib.rs
RUN cargo build --release && \
    cargo build --package healthscope --release && \
    rm -rf ./target/release/.fingerprint/rudr-* && \
    rm -rf ./target/release/.fingerprint/healthscope-*

# Build real binaries now
COPY ./src ./src
COPY ./healthscope/src ./healthscope/src
RUN cargo build --release
RUN cargo build --release --package healthscope

FROM ${BASE_IMAGE}
RUN apt-get update && apt-get install -y pkg-config libssl-dev openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/app
ARG PACKAGE_NAME
COPY --from=builder /usr/src/rudr/target/release/${PACKAGE_NAME} .
ENV RUST_LOG ${PACKAGE_NAME}=info
RUN echo "./${PACKAGE_NAME}" > entrypoint.sh
RUN chmod 0755 entrypoint.sh
ENTRYPOINT ["/bin/sh", "./entrypoint.sh"]
