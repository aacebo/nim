FROM rust:1.99-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    pkg-config \
    libssl-dev \
    g++ \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

ENV OPENSSL_STATIC=1
