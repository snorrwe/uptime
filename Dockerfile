# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine AS builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen pkgconfig libressl-dev

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.32/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work

COPY ./package-lock.json ./package.json ./
RUN npm install

COPY . .

RUN cargo leptos build --release -vv

FROM alpine AS runner

WORKDIR /app

COPY --from=builder /work/target/release/uptime /app/
COPY --from=builder /work/target/tmp /app/
COPY --from=builder /work/target/site /app/
COPY --from=builder /work/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=/app/site
ENV DATABASE_URL=/var/uptime/uptime.db
EXPOSE 8080

CMD ["/app/uptime", "--config", "/etc/uptime/uptime.toml"]

