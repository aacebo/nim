prepare:
    docker compose pull --ignore-buildable
    docker compose build --pull
    docker compose run --rm --no-deps gateway rustc --version
    docker compose run --rm --no-deps gateway cargo fetch --locked
    docker compose run --rm --no-deps gateway cargo build --workspace --all-features --release --locked

start:
    docker compose -f compose.yaml -f compose.offline.yaml up -d --no-build --pull never

stop:
    docker compose down
