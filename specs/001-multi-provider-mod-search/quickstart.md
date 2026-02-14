# Quickstart Guide: MCM-Finder Development

**Feature**: 001-multi-provider-mod-search  
**Date**: 2026-02-13  
**Status**: Implementation-Ready

## Overview

This guide provides step-by-step instructions for setting up the MCM-Finder development environment and implementing the multi-provider mod search tool.

---

## Prerequisites

### Required Tools
- **Rust 1.75+**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **SQLite 3.35+**: Pre-installed on most systems; verify with `sqlite3 --version`
- **Node.js 18+** (for frontend): `nvm install 18`
- **Git**: For version control

### Optional Tools
- **cargo-watch**: Auto-rebuild on changes: `cargo install cargo-watch`
- **sqlx-cli**: Database migrations: `cargo install sqlx-cli`

---

## Project Structure Setup

### 1. Initialize Repository Structure

```bash
# Create backend structure
mkdir -p backend/src/{models,providers,services,api,cli}
mkdir -p backend/tests/{contract,integration,unit}
mkdir -p backend/migrations

# Create frontend structure
mkdir -p frontend/src/{components,pages,services}
mkdir -p frontend/tests/{e2e,component}

# Create data directory for SQLite databases
mkdir -p data

# Create documentation directory
mkdir -p backend/docs
```

### 2. Initialize Rust Backend

```bash
cd backend

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "mcm-finder"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# HTTP client
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Web framework
axum = "0.7"
tower = { version = "0.4", features = ["util", "timeout"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }

# CLI
clap = { version = "4.4", features = ["derive"] }
indicatif = "0.17"
colored = "2.1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"  # Benchmarking
mockall = "0.12"   # Mocking for tests

[[bin]]
name = "mcm-server"
path = "src/api/main.rs"

[[bin]]
name = "mcm"
path = "src/cli/main.rs"
EOF
```

### 3. Initialize Frontend (React + TypeScript)

```bash
cd ../frontend
npx create-react-app . --template typescript

# Install dependencies
npm install axios react-query
npm install -D @types/react @types/node
```

---

## Development Workflow

### Phase 1: Provider Adapters

**Goal**: Implement Modrinth and CurseForge adapters with normalization.

#### 1.1 Create Provider Trait

```bash
# backend/src/providers/mod.rs
cat > src/providers/mod.rs << 'EOF'
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod modrinth;
pub mod curseforge;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    Modrinth,
    CurseForge,
}

#[derive(Debug, Clone)]
pub struct ProviderResult {
    pub source: Provider,
    pub provider_mod_id: String,
    pub name: String,
    pub slug: String,
    pub author: String,
    pub summary: String,
    pub supported_versions: Vec<String>,
    pub supported_loaders: Vec<String>,
    pub downloads: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub relevance_score: f64,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    async fn search(&self, query: &str, filters: Option<Filters>) -> Result<Vec<ProviderResult>, ProviderError>;
    fn provider_name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
EOF
```

#### 1.2 Implement Modrinth Adapter

```bash
# backend/src/providers/modrinth.rs
cat > src/providers/modrinth.rs << 'EOF'
use super::{Provider, ProviderAdapter, ProviderResult, ProviderError};
use async_trait::async_trait;
use serde::Deserialize;

pub struct ModrinthAdapter {
    client: reqwest::Client,
    base_url: String,
}

impl ModrinthAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }
}

#[async_trait]
impl ProviderAdapter for ModrinthAdapter {
    async fn search(&self, query: &str, _filters: Option<Filters>) -> Result<Vec<ProviderResult>, ProviderError> {
        let url = format!("{}/search?query={}", self.base_url, query);
        let response = self.client.get(&url).send().await?;
        
        if response.status() == 429 {
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }
        
        let search_response: ModrinthSearchResponse = response.json().await?;
        
        Ok(search_response.hits.into_iter().map(|hit| ProviderResult {
            source: Provider::Modrinth,
            provider_mod_id: hit.project_id,
            name: hit.title,
            slug: hit.slug,
            author: hit.author,
            summary: hit.description,
            supported_versions: hit.versions,
            supported_loaders: hit.categories.iter()
                .filter(|c| ["fabric", "forge", "quilt", "neoforge"].contains(&c.as_str()))
                .cloned()
                .collect(),
            downloads: hit.downloads,
            last_updated: chrono::DateTime::parse_from_rfc3339(&hit.date_modified)
                .unwrap()
                .with_timezone(&chrono::Utc),
            relevance_score: 1.0, // TODO: Implement scoring
        }).collect())
    }
    
    fn provider_name(&self) -> &str {
        "modrinth"
    }
}

#[derive(Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
}

#[derive(Deserialize)]
struct ModrinthHit {
    project_id: String,
    title: String,
    slug: String,
    author: String,
    description: String,
    categories: Vec<String>,
    versions: Vec<String>,
    downloads: u64,
    date_modified: String,
}
EOF
```

**Contract Test**:
```bash
# backend/tests/contract/modrinth_test.rs
mkdir -p tests/contract
cat > tests/contract/modrinth_test.rs << 'EOF'
use mcm_finder::providers::{modrinth::ModrinthAdapter, ProviderAdapter};

#[tokio::test]
async fn test_modrinth_search_returns_normalized_schema() {
    let adapter = ModrinthAdapter::new();
    let results = adapter.search("sodium", None).await.unwrap();
    
    assert!(!results.is_empty());
    for result in results {
        assert!(!result.name.is_empty());
        assert!(!result.slug.is_empty());
        assert!(!result.author.is_empty());
        assert!(result.relevance_score >= 0.0 && result.relevance_score <= 1.0);
    }
}
EOF
```

#### 1.3 Run Contract Tests

```bash
cargo test --test modrinth_test
```

**Expected Output**:
```
running 1 test
test test_modrinth_search_returns_normalized_schema ... ok
```

---

### Phase 2: Consolidation Service

**Goal**: Merge provider results with deduplication and conflict detection.

#### 2.1 Create Consolidation Service

```bash
# backend/src/services/consolidation.rs
cat > src/services/consolidation.rs << 'EOF'
use crate::providers::ProviderResult;
use std::collections::HashMap;

pub struct ConsolidationService;

impl ConsolidationService {
    pub fn consolidate(results: Vec<ProviderResult>) -> Vec<ConsolidatedMod> {
        let mut grouped: HashMap<String, Vec<ProviderResult>> = HashMap::new();
        
        // Group by canonical slug (normalized name)
        for result in results {
            let canonical_slug = Self::normalize_slug(&result.slug);
            grouped.entry(canonical_slug).or_default().push(result);
        }
        
        // Convert groups to consolidated profiles
        grouped.into_iter().map(|(slug, provider_records)| {
            ConsolidatedMod {
                canonical_slug: slug,
                canonical_name: provider_records[0].name.clone(),
                provider_records,
                conflicts: vec![], // TODO: Implement conflict detection
            }
        }).collect()
    }
    
    fn normalize_slug(slug: &str) -> String {
        slug.to_lowercase().replace("-", "").replace("_", "")
    }
}

#[derive(Debug)]
pub struct ConsolidatedMod {
    pub canonical_slug: String,
    pub canonical_name: String,
    pub provider_records: Vec<ProviderResult>,
    pub conflicts: Vec<String>,
}
EOF
```

---

### Phase 3: REST API Server

**Goal**: Expose search endpoints via axum.

#### 3.1 Create API Server

```bash
# backend/src/api/main.rs
mkdir -p src/api
cat > src/api/main.rs << 'EOF'
use axum::{
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/search", post(search_mods));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on http://{}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn search_mods(Json(req): Json<SearchRequest>) -> Json<SearchResponse> {
    // TODO: Implement actual search
    Json(SearchResponse {
        data: vec![],
        metadata: SearchMetadata {
            total_results: 0,
            search_duration_ms: 0,
        },
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Deserialize)]
struct SearchRequest {
    keywords: Vec<String>,
}

#[derive(Serialize)]
struct SearchResponse {
    data: Vec<String>,
    metadata: SearchMetadata,
}

#[derive(Serialize)]
struct SearchMetadata {
    total_results: u32,
    search_duration_ms: u64,
}
EOF
```

#### 3.2 Run Server

```bash
cargo run --bin mcm-server
```

**Test**:
```bash
curl http://localhost:3000/api/v1/health
# Expected: {"status":"healthy","version":"0.1.0"}
```

---

### Phase 4: CLI Client

**Goal**: Create CLI that consumes backend services.

#### 4.1 Create CLI

```bash
# backend/src/cli/main.rs
cat > src/cli/main.rs << 'EOF'
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "mcm")]
#[command(about = "Minecraft Mod Finder CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        #[arg(help = "Search keywords")]
        query: String,
        
        #[arg(short, long, help = "Minecraft version filter")]
        version: Option<String>,
        
        #[arg(short, long, help = "Loader filter (fabric/forge/quilt)")]
        loader: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Search { query, version, loader } => {
            println!("{}", "Searching for mods...".bright_blue());
            println!("Query: {}", query);
            if let Some(v) = version {
                println!("Version: {}", v);
            }
            if let Some(l) = loader {
                println!("Loader: {}", l);
            }
            
            // TODO: Call backend services
            println!("{}", "✓ Found 0 mods".green());
        }
    }
}
EOF
```

#### 4.2 Run CLI

```bash
cargo run --bin mcm -- search "world generation" --version 1.20.1 --loader fabric
```

**Expected Output**:
```
Searching for mods...
Query: world generation
Version: 1.20.1
Loader: fabric
✓ Found 0 mods
```

---

## Database Setup

### 1. Create Migration

```bash
sqlx database create --database-url sqlite:data/sessions.db

# Create migrations
mkdir -p migrations
cat > migrations/001_create_sessions.sql << 'EOF'
CREATE TABLE searches (
    id TEXT PRIMARY KEY,
    query_hash TEXT NOT NULL,
    query_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_query_hash ON searches(query_hash);
EOF

sqlx migrate run --database-url sqlite:data/sessions.db
```

---

## Testing Strategy

### Unit Tests
```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test providers::modrinth
```

### Contract Tests
```bash
# Run provider normalization tests
cargo test --test contract
```

### Integration Tests
```bash
# Run end-to-end search workflow tests
cargo test --test integration
```

### Performance Benchmarks
```bash
# Run performance benchmarks
cargo bench
```

### Acceptance Evaluation Protocol (NFR-004 / NFR-005)

- **Target sample**: at least 20 professional users (modpack creators, server operators, technical reviewers).
- **Scenario set (required for each participant)**:
  1. Build shortlist for a modern target version (ex: `1.20.1`) with at least one loader filter.
  2. Build shortlist for a legacy target version (ex: `1.7.10`) with version-aware sorting enabled.
- **Measurement method**:
  - Completion success (`yes/no`) for each scenario.
  - End-to-end completion time from first query to final shortlist save.
  - Usefulness rating (`1-5`) after both scenarios.
- **Data capture template per participant**:
  - `participant_id`, `first_time_user`, `scenario_1_success`, `scenario_1_time_s`, `scenario_2_success`,
    `scenario_2_time_s`, `overall_usefulness_rating`.
- **Pass criteria**:
  - At least 90% of participants rate usefulness at **4/5 or above** (NFR-004).
  - At least 85% of **first-time** participants complete the workflow within **5 minutes** (NFR-005).
- **Protocol maintenance**:
  - Re-run this evaluation after major ranking/filtering/discovery changes.
  - Keep sample size and rating thresholds unchanged unless spec.md NFRs are formally updated.

---

## Development Commands

### Backend
```bash
# Auto-rebuild on changes
cargo watch -x 'run --bin mcm-server'

# Run with debug logging
RUST_LOG=debug cargo run --bin mcm-server

# Format code
cargo fmt

# Lint
cargo clippy
```

### Frontend
```bash
cd frontend

# Development server
npm start

# Build for production
npm run build

# Run tests
npm test
```

---

## Validation Checklist

Before marking Phase 1 complete:

- [ ] Modrinth adapter returns normalized schema (contract test passes)
- [ ] CurseForge adapter returns normalized schema (contract test passes)
- [ ] Consolidation service merges results without duplicates
- [ ] API server responds to /health endpoint
- [ ] API server accepts POST /search requests
- [ ] CLI parses search command with filters
- [ ] SQLite database created with sessions table
- [ ] All cargo tests pass (`cargo test`)

---

## Constitution Compliance Review

Run this checklist before release to confirm feature-level compliance:

- [ ] **Correctness Before Convenience**: Validate conflict detection, degraded-mode behavior, and discovery-state transitions against `spec.md` and `contracts/`.
- [ ] **Mandatory Verification First**: Execute contract, integration, and benchmark commands listed in this guide; record evidence in CI logs.
- [ ] **Explicit Failure and Security Handling**: Verify API validation, rate limits, and secret-loading paths using negative test cases.
- [ ] **Minimal Complexity and Maximum Reuse**: Confirm CLI/Web flows reuse shared backend services with no duplicated business logic.
- [ ] **Deterministic Delivery and Traceability**: Confirm `tasks.md` completion and unresolved blockers are explicitly documented.

If local Rust tooling is unavailable, defer command execution to CI and keep release gate open until evidence is collected.

---

## Next Steps

After Phase 1 completion:

1. **Phase 2 (Tasks)**: Run `/speckit.tasks` to generate implementation task breakdown
2. **Circuit Breaker**: Implement provider failure isolation
3. **Deep Discovery**: Add relationship pattern detection
4. **Frontend**: Build React search UI
5. **Deployment**: Package binaries for Linux/macOS/Windows

---

## Troubleshooting

### Common Issues

**Issue**: `cargo build` fails with SSL error  
**Solution**: Use rustls instead of native-tls: `cargo add reqwest --features rustls-tls --no-default-features`

**Issue**: SQLite database locked  
**Solution**: Enable WAL mode: `PRAGMA journal_mode=WAL;`

**Issue**: Provider API rate limited  
**Solution**: Implement multi-layer caching (provider 24h TTL + consolidated query cache + client-local session cache with freshness checks; see research.md)

---

## References

- **Spec**: `/specs/001-multi-provider-mod-search/spec.md`
- **Research**: `/specs/001-multi-provider-mod-search/research.md`
- **Data Model**: `/specs/001-multi-provider-mod-search/data-model.md`
- **API Contracts**: `/specs/001-multi-provider-mod-search/contracts/`
- **Modrinth API**: https://docs.modrinth.com/api-spec/
- **CurseForge API**: https://docs.curseforge.com/
