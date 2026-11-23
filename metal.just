all:
    #!/usr/bin/env bash
    set -euo pipefail
    pids=()
    cleanup() {
        status=$?
        trap - EXIT INT TERM
        for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done
        for pid in "${pids[@]}"; do wait "$pid" 2>/dev/null || true; done
        exit "$status"
    }
    trap cleanup EXIT
    trap 'exit 130' INT TERM
    if listener=$(lsof -nP -iTCP:9999 -sTCP:LISTEN); then
        printf '%s\n' "$listener" >&2
        exit 1
    fi
    PORT=9999 target/release/nim-gateway &
    gateway=$!
    pids+=("$gateway")
    while ! curl -fsS http://127.0.0.1:9999/health >/dev/null; do
        if ! kill -0 "$gateway" 2>/dev/null; then wait "$gateway"; exit $?; fi
        sleep 0.25
    done
    target/release/nim-sleep
    for service in nim-encode nim-exec nim-text nim-audio nim-vision; do
        "target/release/$service" &
        pids+=("$!")
    done
    wait "$gateway"

up:
    docker compose up -d postgres nats prometheus loki tempo grafana alloy
    cargo build --workspace --release --locked
    just metal all
