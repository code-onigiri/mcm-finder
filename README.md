# MCM-Finder - Multi-Provider Minecraft Mod Search

MCM-Finder is a Rust + React tool for searching Minecraft mods across Modrinth and CurseForge, applying advanced filters, and generating composite relationship insights.

## Prerequisites

- Rust 1.75+
- Node.js 18+
- SQLite 3.35+

## Installation

```bash
# Backend
cd backend
cargo build

# Frontend
cd ../frontend
npm install
```

## Configuration

Set environment variables before running CLI/API:

| Variable | Required | Description |
|---|---|---|
| `CURSEFORGE_API_KEY` | Recommended | Enables CurseForge provider requests. |
| `BIND_ADDRESS` | Optional | API bind host (default `127.0.0.1`; use `0.0.0.0` in containers). |
| `FRONTEND_ORIGIN` | Optional | CORS allow-origin(s) for API server (comma-separated, default `http://localhost:3000,http://localhost:3001`). |
| `API_RATE_LIMIT_REQUESTS` | Optional | Max requests per client per window (default `120`). |
| `API_RATE_LIMIT_WINDOW_SECONDS` | Optional | Rate-limit window duration (default `60`). |
| `REACT_APP_API_BASE_URL` | Optional (frontend) | API base URL for web UI (default `http://localhost:3000`). |

## Usage Examples

### CLI

```bash
cd backend

# Basic search
cargo run --bin mcm -- search "optimization"

# Filtered search
cargo run --bin mcm -- search "performance" \
  --version 1.20.1 \
  --loader fabric \
  --category utility \
  --provider modrinth \
  --sort update-recency
```

### API Server

```bash
cd backend
cargo run --bin mcm-server
```

```bash
curl -X POST http://localhost:3000/api/v1/search \
  -H "Content-Type: application/json" \
  -d '{
    "keywords": ["performance"],
    "filters": {"minecraft_version": "1.20.1", "loaders": ["fabric"]},
    "provider_scope": ["modrinth", "curseforge"],
    "limit": 20
  }'
```

### Web UI

```bash
cd frontend
npm start
```

Open the URL printed by `npm start` (typically `http://localhost:3001` when backend is on `http://localhost:3000`) and run searches from the UI form.

## Documentation

- [Quickstart + Acceptance Protocol](specs/001-multi-provider-mod-search/quickstart.md)
- [Backend API Docs](backend/docs/api.md)
- [CLI Docs](backend/docs/cli.md)
- [Deployment Guide](backend/docs/deployment.md)
- [Feature Specification](specs/001-multi-provider-mod-search/spec.md)

## License

MIT - see [LICENSE](LICENSE).
