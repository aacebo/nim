FROM rust:1.99-slim-bookworm

RUN apt-get update && \
    apt-get install -y --no-install-recommends cron && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --chmod=0644 bins/nim-sleep/crontab /etc/cron.d/nim-sleep

CMD ["sh", "-c", "printenv > /etc/environment && cargo build -p nim-sleep --release && exec cron -f"]
