# MCM-Finder Deployment Guide

## 1) Build Backend Binaries

```bash
cd backend

# Native build
cargo build --release

# Explicit Linux target
cargo build --release --target x86_64-unknown-linux-gnu
```

Cross-platform targets (require installed Rust targets/toolchains):

```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc
```

## 2) Build Frontend Bundle

```bash
cd frontend
npm install
npm run build
```

Deploy static output from `frontend/build/`.

## 3) Runtime Data Directory

On startup, backend initializes SQLite files under `data/`:
- `sessions.db`
- `cache.db`
- `discovery.db`

Ensure the runtime user has read/write access to this directory.

## 4) Environment Variables

| Variable | Purpose |
|---|---|
| `CURSEFORGE_API_KEY` | CurseForge provider authentication key. |
| `BIND_ADDRESS` | Host/IP for API server bind (use `0.0.0.0` in containers). |
| `FRONTEND_ORIGIN` | CORS origin(s) for browser clients (comma-separated). |
| `API_RATE_LIMIT_REQUESTS` | Max requests per client in window. |
| `API_RATE_LIMIT_WINDOW_SECONDS` | Window size for API throttling. |
| `RUST_LOG` | Runtime logging level (ex: `info`, `debug`). |

## 5) Launch Commands

```bash
# API server
./backend/target/release/mcm-server

# CLI
./backend/target/release/mcm search "performance"
```

## 6) Operational Checks

- `GET /api/v1/health` returns healthy/degraded status.
- Search endpoint returns pagination metadata and provider error details when degraded.
- Rate limiting returns HTTP `429` with `Retry-After`.
