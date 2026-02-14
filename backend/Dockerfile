FROM rust:1.85-bookworm AS builder

WORKDIR /workspace

COPY backend/Cargo.toml backend/Cargo.lock* ./backend/
COPY backend/src ./backend/src
COPY backend/migrations ./backend/migrations

WORKDIR /workspace/backend
RUN cargo build --release --bin mcm-server

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/backend/target/release/mcm-server /usr/local/bin/mcm-server
COPY backend/migrations ./migrations

EXPOSE 3000
ENV RUST_LOG=info

CMD ["mcm-server"]
