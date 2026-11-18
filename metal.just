[parallel]
all: gateway encode exec sleep text audio vision

up:
    docker compose up -d postgres nats prometheus loki tempo grafana alloy
    cargo build --workspace --release --locked
    just --jobs 7 metal all

gateway:
    PORT=9999 target/release/nim-gateway

wait-gateway:
    @attempt=0; while [ "$attempt" -lt 120 ]; do \
        if curl -fsS http://127.0.0.1:9999/health >/dev/null; then exit 0; fi; \
        attempt=$((attempt + 1)); sleep 0.25; \
    done; echo "nim-gateway did not become ready" >&2; exit 1

encode: wait-gateway
    target/release/nim-encode

exec: wait-gateway
    target/release/nim-exec

sleep: wait-gateway
    target/release/nim-sleep

text: wait-gateway
    target/release/nim-text

audio: wait-gateway
    target/release/nim-audio

vision: wait-gateway
    target/release/nim-vision
