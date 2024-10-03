default:
    @just --list

# Run the project locally
watch $RUST_BACKTRACE="1":
    cargo leptos watch

# Run tests (backend & frontend)
test:
    cargo watch -- cargo leptos test

prepare $DATABASE_URL="sqlite://uptime.db":
    cargo sqlx prepare

fmt:
    cargo fmt
    leptosfmt **/*rs
    just --unstable --fmt

add_migration name:
    sqlx migrate add -r {{ name }}

setup-db $DATABASE_URL="sqlite://uptime.db":
    sqlx database setup
    @just prepare

init:
    npm i

docker-build:
    docker build . -t uptime

docker-run: docker-build
    docker run --rm -it -v ./uptime.toml:/app/uptime.toml -p8080 uptime
