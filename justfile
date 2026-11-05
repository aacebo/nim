prepare-offline:
    docker compose pull --ignore-buildable
    docker compose build --pull
    docker compose run --rm --no-deps gateway rustc --version
    docker compose run --rm --no-deps gateway cargo fetch --locked
    docker compose run --rm --no-deps gateway cargo build --workspace --all-features --release --locked
