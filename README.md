# MCM-Finder - Multi-Provider Minecraft Mod Search

Professional Minecraft mod search tool that aggregates results from multiple providers (Modrinth and CurseForge) with advanced filtering, version-aware compatibility ranking, and deep relationship discovery.

## Features

- **Multi-Provider Search**: Unified search across Modrinth and CurseForge
- **Version-Aware Filtering**: Filter by Minecraft version, mod loaders, categories
- **Deep Discovery**: Identify mod relationships, dependencies, and conflicts
- **Dual Interface**: Both CLI and Web UI with equivalent functionality
- **Session Preservation**: Save and share search results for team review

## Quick Start

See [Quick Start Guide](specs/001-multi-provider-mod-search/quickstart.md) for detailed setup and usage instructions.

### Prerequisites

- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Node.js 18+ (for web UI)
- SQLite 3.35+

### Backend (CLI + API Server)

```bash
cd backend
cargo build --release

# Run CLI
cargo run --bin mcm -- search "performance mods" --version 1.20.1 --loader fabric

# Run API server
cargo run --bin mcm-server
```

### Frontend (Web UI)

```bash
cd frontend
npm install
npm start
```

## Architecture

- **Backend**: Rust with Tokio async runtime, Axum web framework, SQLite storage
- **Frontend**: React + TypeScript, consuming backend REST API
- **Storage**: SQLite with WAL mode for sessions and caching

## Documentation

- [Specification](specs/001-multi-provider-mod-search/spec.md)
- [Implementation Plan](specs/001-multi-provider-mod-search/plan.md)
- [Data Model](specs/001-multi-provider-mod-search/data-model.md)
- [API Contracts](specs/001-multi-provider-mod-search/contracts/)
- [Quick Start](specs/001-multi-provider-mod-search/quickstart.md)

## Status

🚧 Under active development - Feature 001: Multi-Provider Mod Search

## License

MIT - See [LICENSE](LICENSE) for details.
