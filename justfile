default:
    @just --list

# Run the project locally
watch $RUST_BACKTRACE="1":
    cargo leptos watch

# Run tests (backend & frontend)
test:
    cargo watch -- cargo leptos test
