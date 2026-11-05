FROM rust:1.99.1-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    g++ \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy

WORKDIR /app

CMD ["cargo", "run", "-p", "nim-vision", "--release", "--locked"]
