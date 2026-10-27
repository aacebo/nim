FROM rust:1.99-slim-bookworm

RUN apt-get update && \
    apt-get install -y --no-install-recommends cron && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --chmod=0644 bins/nim-sleep/crontab /etc/cron.d/nim-sleep

CMD ["sh", "-c", "printenv | sed -n '/^DATABASE_URL=/p; /^NATS_URL=/p; /^RUST_LOG=/p' > /etc/environment && exec cron -f"]
