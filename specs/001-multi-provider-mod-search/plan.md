# Implementation Plan: Professional Multi-Provider Mod Finder

**Branch**: `001-multi-provider-mod-search` | **Date**: 2026-02-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-multi-provider-mod-search/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a professional Minecraft mod search tool that aggregates results from multiple providers (Modrinth and CurseForge initially) with advanced filtering, version-aware compatibility ranking, deep relationship discovery, and composite information views. The tool provides equivalent functionality through both CLI and Web UI interfaces, enabling professional modpack creators and server operators to efficiently evaluate and shortlist mods.

## Technical Context

**Language/Version**: Rust 1.75+ (zero-cost abstractions, native concurrency, compile-time schema validation)  
**Primary Dependencies**: reqwest (HTTP), tokio (async), serde (normalization), sqlx (SQLite), axum (REST API), clap (CLI)  
**Storage**: SQLite with WAL mode (sessions, provider/discovery cache) + in-memory cache + client-local session cache  
**Testing**: cargo test (unit/integration/contract tests), criterion (performance benchmarks)  
**Target Platform**: Cross-platform (Linux/macOS/Windows for CLI, web browsers for UI)
**Project Type**: Web application (CLI + Web UI with shared backend)  
**Performance Goals**: 95% of searches complete within 30s for 200 candidates (NFR-001)  
**Constraints**: Degraded mode notification within 10s on provider failure (NFR-003), 99% consistent top-20 ordering (NFR-002), repeated-query cache acceleration (NFR-006)  
**Scale/Scope**: Multi-provider aggregation (2 required initially, 4 phased), deep discovery computation, composite view generation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Initial Check (Pre-Phase 0):**
- [x] **Correctness Before Convenience**: Behavior deltas defined in spec (multi-provider aggregation, version-aware filtering, deep discovery). Edge cases documented (provider unavailable, conflicting metadata, sparse results, ambiguous signals). Failure modes specified (degraded coverage notice, discovery progress visibility).
- [x] **Mandatory Verification First**: Acceptance scenarios defined per user story with independent test descriptions. Contract tests needed for provider normalization. Integration tests required for consolidated search. Performance benchmarks needed for NFR-001/002/003.
- [x] **Explicit Failure and Security Handling**: Provider API input validation required. Error propagation for network failures specified (degraded mode). No credential storage in spec scope (public APIs). Rate limiting and abuse mitigation documented and resolved.
- [x] **Minimal Complexity and Maximum Reuse**: New project (no existing code to reuse). Provider abstraction justified by multi-provider requirement. Shared backend for CLI/Web justified by consistency requirement (FR-001). Result normalization layer required for consolidation (FR-002).
- [x] **Deterministic Delivery and Traceability**: Rollback: feature toggle for new providers during phased expansion. Risk: provider API changes require adapter updates. Validation: search consistency tests, performance benchmarks, acceptance task completion timing.

**Post-Phase 1 Re-Check:**
- [x] **Correctness Before Convenience**: Data model defines validation rules per entity. API contracts specify error responses for edge cases. Conflict detection algorithm documented in provider-normalization.md.
- [x] **Mandatory Verification First**: Contract tests specified for provider normalization (10 test cases). Integration tests defined for search workflow. Performance benchmarks defined with <30s SLA. Quickstart includes validation checklist.
- [x] **Explicit Failure and Security Handling**: Rate limiting strategy defined (exponential backoff, circuit breaker). Input validation rules in data-model.md. Error aggregation pattern in API contracts. No secrets required (public APIs). **RESOLVED**: Rate limiting implemented via per-IP throttling + provider circuit breaker.
- [x] **Minimal Complexity and Maximum Reuse**: Provider adapter pattern reuses reqwest/tokio ecosystem. No unnecessary abstractions introduced. Shared backend services eliminate CLI/Web duplication.
- [x] **Deterministic Delivery and Traceability**: Validation commands in quickstart.md (cargo test, cargo bench). Risk mitigation documented in research.md. Rollback strategy: provider feature toggles. All FR requirements mapped to contracts/data-model.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Web application structure (CLI + Web UI with shared backend)
backend/
├── src/
│   ├── models/          # Data models: SearchQuery, ModProfile, DiscoveryEvidence
│   ├── providers/       # Provider adapters: modrinth.py/rs, curseforge.py/rs
│   ├── services/        # Core services: aggregation, filtering, discovery, normalization
│   ├── api/             # REST/GraphQL API endpoints
│   └── cli/             # CLI command handlers (reuses services)
└── tests/
    ├── contract/        # Provider normalization contracts
    ├── integration/     # End-to-end search workflows
    └── unit/            # Service and model unit tests

frontend/
├── src/
│   ├── components/      # Search form, result list, mod detail, filter panel
│   ├── pages/           # Search page, detail page, comparison view
│   └── services/        # API client for backend
└── tests/
    ├── e2e/             # Acceptance scenario automation
    └── component/       # Component behavior tests
```

**Structure Decision**: Web application structure selected. Backend provides shared services for both CLI and Web UI to ensure equivalent functionality (FR-001). CLI consumes backend services directly without HTTP overhead. Frontend communicates with backend via API for web-based search workflow.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**No violations identified.** All complexity is justified by explicit requirements:
- Multi-provider aggregation required by FR-002, FR-010
- Shared backend required by FR-001 (equivalent CLI/Web workflows)
- Deep discovery required by FR-006 (relationship patterns)
- Deduplication layer required by consolidation (FR-008)

## Phase 0-1 Artifacts Generated

**Phase 0 (Research)**:
- ✅ `research.md`: Language choice (Rust), storage strategy (SQLite), API patterns (circuit breaker, deduplication)
- ✅ Live API verification snapshot: Modrinth search reachable (200), CurseForge requires `x-api-key` (403 without key)

**Phase 1 (Design)**:
- ✅ `data-model.md`: 5 core entities with validation rules, relationships, state transitions
- ✅ `contracts/rest-api.md`: 7 REST endpoints with request/response schemas, error codes, rate limiting
- ✅ `contracts/provider-normalization.md`: Modrinth/CurseForge API mappings, conflict detection rules, 10 test cases
- ✅ `quickstart.md`: Development setup, implementation phases, validation checklist

**Agent Context**:
- ✅ Updated `.github/agents/copilot-instructions.md` with Rust, tokio, axum, SQLite stack

**Next Steps**:
- Run `/speckit.tasks` to generate implementation task breakdown (Phase 2)
- Begin implementation following quickstart.md phases
- Execute contract tests for provider adapters
