FROM rust:1.99.1-slim-bookworm

RUN apt-get update && \
    apt-get install -y --no-install-recommends cron pkg-config libssl-dev ca-certificates g++ && \
    rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy

WORKDIR /app

COPY --chmod=0644 bins/nim-sleep/crontab /etc/cron.d/nim-sleep

CMD ["sh", "-c", "printenv > /etc/environment && cargo build -p nim-sleep --release --locked && exec cron -f"]
