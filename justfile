default:
    @just --list

export DATABASE_URL := "sqlite://uptime.db"

# Run the project locally
watch $RUST_BACKTRACE="1":
    cargo leptos watch

# Run tests (backend & frontend)
test:
    cargo leptos test
    cargo leptos end-to-end

prepare:
    cargo sqlx prepare

fmt:
    cargo fmt
    leptosfmt **/*rs
    just --unstable --fmt

add_migration name:
    sqlx migrate add -r {{ name }}

setup-db:
    sqlx database setup
    @just prepare

init:
    npm i

docker-build:
    docker build . -t uptime

docker-run: docker-build
    docker run --rm -it -v ./uptime.toml:/app/uptime.toml -p8080:8080 uptime
