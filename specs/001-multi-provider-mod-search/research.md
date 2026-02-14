# Research: Professional Multi-Provider Mod Finder

**Feature**: 001-multi-provider-mod-search  
**Date**: 2026-02-13  
**Phase**: Phase 0 - Technical Research

## Overview

This document captures research findings and technical decisions for the MCM-Finder tool, resolving all NEEDS CLARIFICATION items from the Technical Context section of the implementation plan.

## 1. Language & Runtime Selection

### Decision: Rust 1.75+

**Rationale:**
- **Performance Requirements (NFR-001)**: 95% of searches must complete within 30s for 200 candidates. Rust's zero-cost abstractions and native concurrency guarantee consistent sub-100ms aggregation per provider, whereas Python's GIL limits true parallelism even with asyncio
- **Type Safety**: Serde provides compile-time validation of provider schema transformations, critical for FR-002 (consolidated result consistency) and FR-008 (conflict detection). Python's Pydantic defers validation to runtime
- **Concurrency Model**: Tokio runtime proven for thousands of concurrent connections with structured concurrency preventing task leakage during provider timeouts
- **Deployment Simplicity**: Single binary for cross-platform CLI (Linux/macOS/Windows) vs Python's packaging complexity
- **CLI Performance**: No interpreter startup delay (~50-100ms saved per invocation) for rapid iterative searches

**Alternatives Considered:**
- **Python 3.11+**: Faster initial development (~20% velocity gain) but risks NFR-001/002 compliance. GIL bottleneck with 200 concurrent requests, runtime schema validation risks, and interpreter overhead for CLI workflow

**Technology Stack:**
```
Backend (Shared Services):
- reqwest: Async HTTP client with connection pooling and retry logic
- tokio: Async runtime with sync primitives for concurrent provider requests
- serde/serde_json: Provider response normalization with compile-time validation
- sqlx: Async database driver for SQLite
- axum: REST API framework with tower middleware for error handling
- tracing: Structured logging for provider failure diagnostics

CLI:
- clap: Argument parsing with derive macros
- indicatif: Progress bars for deep discovery phase
- colored: Terminal output formatting

Frontend (Separate):
- TypeScript/React consuming REST API from Rust backend
```

**Performance Guarantees:**
- Provider response budget: 8s per provider × 2 providers in parallel = 8s wall-clock
- Consolidation + sorting: 1-2s
- Deep discovery (async): 15-20s
- **Total: ~30s including discovery** ✓

## 2. Storage Strategy

### Decision: SQLite with WAL Mode + In-Memory Cache (Hybrid)

**Rationale:**
- **Deployment Simplicity**: Single-file embedded database, no external service dependencies. All stores in `./data/` directory
- **Concurrent Access**: WAL mode enables unlimited readers + 1 writer, sufficient for CLI + Web UI concurrent usage
- **Structured Queries**: Fast cache invalidation and session retrieval (FR-011) with indexed lookups
- **Performance**: <5ms session lookup, <10ms provider cache hit
- **Persistence**: Session preservation for comparison/review workflow (FR-012)

**Schema Design:**

```sql
-- sessions_db.sqlite
CREATE TABLE searches (
    id TEXT PRIMARY KEY,
    user_ip TEXT,
    query_hash TEXT,
    query_json TEXT,
    results_json TEXT,
    created_at INTEGER,
    expires_at INTEGER
);
CREATE INDEX idx_user_searches ON searches(user_ip, created_at);
CREATE INDEX idx_query_hash ON searches(query_hash);

CREATE TABLE session_mods (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    provider TEXT,
    mod_id TEXT,
    details_json TEXT,
    comparison_notes TEXT,
    FOREIGN KEY(session_id) REFERENCES searches(id)
);

-- cache_db.sqlite
CREATE TABLE provider_responses (
    cache_key TEXT PRIMARY KEY,
    provider TEXT,
    response_json TEXT,
    etag TEXT,
    cached_at INTEGER,
    expires_at INTEGER
);
CREATE INDEX idx_cache_expiry ON provider_responses(provider, expires_at);

-- discovery_cache.sqlite
CREATE TABLE relationships (
    id INTEGER PRIMARY KEY,
    mod_id_a TEXT,
    mod_id_b TEXT,
    rel_type TEXT,
    confidence REAL,
    evidence_json TEXT,
    computed_at INTEGER
);
CREATE INDEX idx_relationships ON relationships(mod_id_a, rel_type);

CREATE TABLE patterns (
    pattern_hash TEXT PRIMARY KEY,
    pattern_data TEXT,
    last_seen INTEGER
);
```

**Cache Strategy:**
- **Cache key**: `sha256(provider + normalized_query + filter_set)`
- **TTL**: 24h for mod search results, 30 days for discovery relationships
- **Invalidation**: Manual on provider API change or TTL expiration
- **Rate limit storage**: Store provider rate-limit headers to respect limits
- **Client-local cache**: Persist recent query results and metadata on each client for fast re-open
- **Client freshness model**: Store `cached_at`, `expires_at`, and `cache_version`; force refresh when
  cache_version mismatch or expiration threshold reached

**Alternatives Considered:**
- **Redis**: Requires external service, doesn't fit "no deployment complexity" constraint, lost on restart
- **JSON files**: No concurrent write safety, poor query performance for cache invalidation
- **RocksDB**: Over-engineered for use case, slower range queries than SQLite
- **PostgreSQL**: Deployment overhead contradicts single-binary approach

## 3. Multi-Provider API Aggregation Architecture

### Decision: Adapter Pattern + Orchestrator with Circuit Breaker

**Rationale:**
- **Provider Abstraction (FR-010)**: Each provider adapter implements `ProviderTrait`, encapsulating API-specific schema and error handling
- **Graceful Degradation (NFR-003)**: Circuit breaker pattern isolates provider failures, enabling search continuation with reduced provider set
- **Extensibility**: GitHub and mc百科 expansion requires only new adapter implementations
- **Consistency (FR-001)**: Shared backend services consumed by both CLI and Web UI

**Architecture Pattern:**

```rust
// Provider abstraction
#[async_trait]
trait Provider {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ProviderResult>>;
    async fn get_mod_details(&self, id: &str) -> Result<ModDetails>;
    fn provider_name(&self) -> &str;
}

// Orchestrator for concurrent aggregation
struct MultiProviderOrchestrator {
    providers: Vec<Box<dyn Provider>>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl MultiProviderOrchestrator {
    async fn search(&self, query: &SearchQuery) -> ConsolidatedResults {
        // Concurrent provider requests with timeout
        let futures: Vec<_> = self.providers.iter()
            .map(|p| timeout(Duration::from_secs(10), p.search(query)))
            .collect();
        
        let results = join_all(futures).await;
        
        // Handle partial failures
        let (successes, failures) = self.partition_results(results);
        
        // Normalize and deduplicate
        self.consolidate(successes, failures)
    }
}
```

**Key Patterns:**

1. **Circuit Breaker** (3-state: CLOSED → OPEN → HALF_OPEN):
   - Failure threshold: 3 consecutive failures
   - Backoff: 30s before recovery test
   - Behavior: If Modrinth circuit opens, continue with CurseForge only

2. **Rate Limit Handling**:
   - Exponential backoff: `min(1000 * 2^attempt, 10000)` ms
   - Track limits from response headers (X-Ratelimit-Remaining)
   - Modrinth: ~300 req/min, CurseForge: ~120 req/min
   - Cache aggregated results (5-min TTL) for shared queries

3. **Result Deduplication** (Multi-level matching):
   - Level 1: Exact match (same slug/canonical ID across providers)
   - Level 2: Fuzzy matching (85% string similarity for name + author)
   - Level 3: Metadata fingerprint (description hash similarity)
   
4. **Conflict Detection** (FR-008):
   - Store all provider versions in consolidated result
   - Flag fields with differing values (e.g., different version compatibility)
   - Present to user: "Version 2.1.0 on Modrinth, 2.0.9 on CurseForge"

5. **Progress Tracking** (FR-007):
   - Non-blocking discovery with event emitter
   - WebSocket/SSE for live updates to Web UI
   - CLI: Progress bar via indicatif
    - States: "Searching providers (✓), discovering relationships (12/47)..."

## 3.1 Live API Verification Snapshot (2026-02-13)

### Verified Calls

1. **Modrinth search endpoint**
   - URL: `https://api.modrinth.com/v2/search?query=sodium&limit=1`
   - Result: **200 OK** with valid JSON payload (`hits`, `offset`, `limit`, `total_hits`)
   - Outcome: Existing normalization assumptions for Modrinth search fields are confirmed against live
     response shape.

2. **CurseForge search endpoint**
   - URL: `https://api.curseforge.com/v1/mods/search?gameId=432&searchFilter=sodium&pageSize=1`
   - Result: **403 Forbidden** without API key
   - Outcome: Contract must require `x-api-key` provisioning and client-safe handling for key absence.

### Verification Conclusion

- Modrinth integration can proceed with live endpoint validation coverage.
- CurseForge integration requires API key onboarding before full live contract testing.
- Planning artifacts now treat API-key management as a first-class dependency and risk item.

**Error Aggregation Strategy:**

```rust
struct SearchResponse<T> {
    data: Vec<T>,
    metadata: SearchMetadata {
        total_time_ms: u64,
        providers_queried: Vec<String>,
        providers_succeeded: Vec<String>,
        provider_errors: Vec<ProviderError>,
        degradation_reason: Option<String>,
    },
}

struct ProviderError {
    provider: String,
    error: String,
    severity: ErrorSeverity, // Warning | Error
    timestamp: u64,
}
```

**User-Facing Notifications:**
- Warning: "CurseForge rate-limited. Showing first 50 results. Try again in 2 minutes."
- Error: "Modrinth connection failed. Showing CurseForge results only (142 mods)."
- Success: "Found 287 mods across Modrinth & CurseForge."

**Provider API Versioning:**
- Adapter tracks API version, auto-detects deprecation
- Graceful fallback: v3 → v2 → v1
- Log warnings on deprecated API usage

**Alternatives Considered:**
- **Service mesh**: Over-engineered for 2-4 providers, adds deployment complexity
- **Message queue**: Unnecessary async infrastructure for synchronous search workflow
- **GraphQL federation**: Providers don't expose GraphQL; REST aggregation simpler

## 4. Testing Strategy

### Decision: Multi-layer testing aligned with Constitution Check

**Rationale:**
- **Mandatory Verification First (Principle II)**: Tests defined before implementation for all features
- **Correctness Before Convenience (Principle I)**: Contract tests enforce provider normalization correctness

**Test Layers:**

1. **Contract Tests** (Provider normalization):
```rust
#[test]
fn all_adapters_return_normalized_schema() {
    for adapter in [modrinth_adapter(), curseforge_adapter()] {
        let result = adapter.search("Quark").await;
        assert!(result.has_field("name"));
        assert!(result.has_field("author"));
        assert!(result.has_field("source")); // Provider identifier
    }
}
```

2. **Integration Tests** (End-to-end search workflows):
```rust
#[test]
fn consolidated_search_merges_providers() {
    let results = orchestrator.search("JourneyMap").await;
    let journeymap = results.iter().find(|m| m.name.contains("Journey"));
    assert!(journeymap.sources.contains(&"modrinth"));
    assert!(journeymap.sources.contains(&"curseforge"));
}
```

3. **Performance Benchmarks** (NFR validation):
```rust
#[test]
fn search_completes_within_30s_for_200_candidates() {
    let start = Instant::now();
    let results = orchestrator.search(heavy_query).await;
    assert!(start.elapsed() < Duration::from_secs(30));
    assert!(results.len() <= 200);
}
```

4. **Acceptance Tests** (User story validation):
- US1: Execute same query in CLI and Web UI, verify identical results
- US2: Search with Minecraft version filter, verify only compatible mods returned
- US3: Execute deep discovery, verify composite profile includes relationships

## 5. Implementation Dependencies

**Build & Development:**
- `cargo` (Rust build tool and package manager)
- `cargo-watch` (Auto-rebuild on file changes for development)
- `sqlx-cli` (Database migration tool)

**Runtime Dependencies:**
- SQLite 3.35+ (WAL mode support)
- No external services required (embedded database)

**CI/CD:**
- Contract tests on provider schema changes
- Performance regression tests (30s SLA enforcement)
- Cross-platform binary builds (Linux/macOS/Windows)

**Documentation:**
- OpenAPI spec generation from axum routes (API documentation)
- CLI help text from clap derive macros (--help output)

## Summary

All NEEDS CLARIFICATION items resolved:

| Item | Decision |
|------|----------|
| Language/Version | Rust 1.75+ |
| Primary Dependencies | reqwest, tokio, serde, sqlx, axum, clap |
| Storage | SQLite with WAL + in-memory cache |
| Testing | pytest equivalent: cargo test (unit/integration/contract/performance) |
| Performance Goals | 30s search completion via concurrent requests + optimized aggregation |
| Rate Limiting | Exponential backoff, circuit breaker, response header tracking |
| API Patterns | Adapter pattern with orchestrator, circuit breaker for isolation |

**Risk Mitigation:**
- Rust learning curve mitigated by bounded domain (mod search) and strong ecosystem documentation
- Provider API changes handled via versioned adapters with fallback logic
- Performance validated via continuous benchmarking against NFR-001/002/003

**Next Steps:**
- Phase 1: Define data models and API contracts
- Phase 1: Create provider adapter stubs
- Phase 1: Implement core orchestrator with circuit breaker
- Phase 1: Generate quickstart documentation
