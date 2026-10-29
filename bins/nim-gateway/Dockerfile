FROM rust:1.99-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    g++ \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["cargo", "run", "-p", "nim-gateway", "--release"]
