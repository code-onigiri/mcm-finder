# Tasks: Professional Multi-Provider Mod Finder

**Feature**: 001-multi-provider-mod-search  
**Input**: Design documents from `/specs/001-multi-provider-mod-search/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Test tasks are required for this feature and must be completed per user story and NFR validation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `- [ ] [ID] [P?] [Story?] Description`

- **Checkbox**: `- [ ]` (required for all tasks)
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3) - only for story-specific tasks
- Include exact file paths in descriptions

## Path Conventions

This is a **Web application** structure (per plan.md):
- Backend: `backend/src/{models,providers,services,api,cli}`
- Frontend: `frontend/src/{components,pages,services}`
- Tests: `backend/tests/{contract,integration,unit}`
- Data: `data/` (SQLite databases)

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Initialize Rust backend and React frontend with required dependencies

- [X] T001 Create directory structure per plan.md: backend/src/{models,providers,services,api,cli}, backend/tests/{contract,integration,unit}, frontend/src/{components,pages,services}, data/
- [X] T002 Initialize Rust backend with Cargo.toml in backend/ directory (dependencies: tokio, reqwest, axum, serde, sqlx, clap, tracing, uuid, chrono, thiserror per research.md)
- [X] T003 [P] Initialize React TypeScript frontend in frontend/ directory (create-react-app with typescript template, add axios and react-query per quickstart.md)
- [X] T004 [P] Configure Rust linting and formatting tools in backend/ (cargo fmt, cargo clippy configuration)
- [X] T005 [P] Create .gitignore for Rust and Node.js artifacts
- [X] T006 [P] Create README.md with project overview and quickstart reference

**Checkpoint**: Project structure initialized - ready for foundational implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Database & Storage Setup

- [X] T007 Create SQLite database schemas in data/ directory: sessions_db.sqlite (searches, session_mods tables per research.md), cache_db.sqlite (provider_responses table), discovery_cache.sqlite (relationships, patterns tables)
- [X] T008 Create database migration scripts in backend/migrations/ for all three database schemas with WAL mode enabled
- [X] T009 [P] Configure sqlx database connection pool in backend/src/db/mod.rs with WAL pragma

### Core Data Models

- [X] T010 [P] Create SearchQueryProfile model in backend/src/models/search_query.rs (fields: id, keywords, filters, provider_scope, sort_mode per data-model.md)
- [X] T011 [P] Create SearchFilters struct in backend/src/models/search_query.rs (minecraft_version, loaders, categories, update_recency_window, min_downloads, open_source_only)
- [X] T012 [P] Create ProviderResultRecord model in backend/src/models/provider_result.rs (all fields from data-model.md with validation rules)
- [X] T013 [P] Create ConsolidatedModProfile model in backend/src/models/consolidated_profile.rs (provider_records, metadata_conflicts, discovery_evidence, compatibility_assessment, maintenance_signals per data-model.md)
- [X] T014 [P] Create DiscoveryEvidenceItem model in backend/src/models/discovery.rs (relationship_type, evidence_source, confidence per data-model.md)
- [X] T015 [P] Create SearchSessionSummary model in backend/src/models/session.rs (original_query, shortlisted_mods, comparison_notes per data-model.md)

### Provider Infrastructure

- [X] T016 Create Provider trait in backend/src/providers/mod.rs with async search and get_mod_details methods per research.md adapter pattern
- [X] T017 [P] Create ProviderResult normalization struct in backend/src/providers/mod.rs with all fields from provider-normalization.md
- [X] T018 [P] Create ProviderError enum in backend/src/providers/mod.rs (Network, RateLimited, InvalidResponse variants)
- [X] T019 [P] Create ModLoader and Provider enums in backend/src/models/enums.rs (Fabric, Forge, Quilt, NeoForge; Modrinth, CurseForge, GitHub, McBaike)

### Error Handling & Logging

- [X] T020 [P] Setup tracing subscriber configuration in backend/src/main.rs with structured logging per research.md
- [X] T021 [P] Create error handling middleware for axum in backend/src/api/middleware/error_handler.rs
- [X] T022 [P] Create custom error types in backend/src/error.rs (DatabaseError, ProviderError, ValidationError)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Unified Professional Search (Priority: P1) 🎯 MVP

**Goal**: Execute one search from CLI or Web UI and return consolidated results from Modrinth and CurseForge

**Independent Test**: Execute the same query in CLI and Web UI; verify both return identical consolidated results

### Provider Adapters for User Story 1

- [X] T023 [P] [US1] Implement Modrinth adapter in backend/src/providers/modrinth.rs (search method, API v2 endpoint mapping per provider-normalization.md test case 1)
- [X] T024 [P] [US1] Implement CurseForge adapter in backend/src/providers/curseforge.rs (search method, API v1 endpoint mapping with x-api-key per provider-normalization.md test case 2)
- [X] T025 [P] [US1] Create Modrinth response deserialization structs in backend/src/providers/modrinth.rs (ModrinthSearchResponse, ModrinthHit per provider-normalization.md)
- [X] T026 [P] [US1] Create CurseForge response deserialization structs in backend/src/providers/curseforge.rs (CurseForgeSearchResponse per provider-normalization.md)
- [X] T027 [P] [US1] Implement Modrinth loader normalization in backend/src/providers/modrinth.rs (map "fabric" → ModLoader::Fabric per provider-normalization.md)
- [X] T028 [P] [US1] Implement CurseForge loader normalization in backend/src/providers/curseforge.rs (map "Fabric" → ModLoader::Fabric per provider-normalization.md)
- [X] T029 [P] [US1] Implement Modrinth version parsing in backend/src/providers/modrinth.rs (validate version format, skip invalid per provider-normalization.md test case 4)
- [X] T030 [P] [US1] Implement CurseForge version extraction in backend/src/providers/curseforge.rs (fetch from /mods/{id}/files endpoint per provider-normalization.md)

### Multi-Provider Orchestration for User Story 1

- [X] T031 [US1] Create MultiProviderOrchestrator in backend/src/services/orchestrator.rs (concurrent provider requests with tokio::join_all per research.md)
- [X] T032 [US1] Implement circuit breaker pattern in backend/src/services/circuit_breaker.rs (3-state: CLOSED/OPEN/HALF_OPEN, 3 failure threshold, 30s backoff per research.md)
- [X] T033 [US1] Implement rate limit handling in backend/src/services/rate_limiter.rs (exponential backoff: min(1000 * 2^attempt, 10000)ms, track X-Ratelimit-Remaining headers per research.md)
- [X] T034 [US1] Integrate circuit breakers into MultiProviderOrchestrator in backend/src/services/orchestrator.rs (one circuit breaker per provider)
- [X] T035 [US1] Implement timeout handling in backend/src/services/orchestrator.rs (10s per provider with tokio::time::timeout)
- [X] T036 [US1] Implement partial failure handling in backend/src/services/orchestrator.rs (partition results into successes/failures, continue with available providers per NFR-003)

### Result Consolidation for User Story 1

- [X] T037 [US1] Create ConsolidationService in backend/src/services/consolidation.rs (merge provider results with deduplication per research.md)
- [X] T038 [US1] Implement mod matching algorithm in backend/src/services/consolidation.rs (slug match + name similarity 0.95 + version overlap per provider-normalization.md)
- [X] T039 [US1] Implement Levenshtein distance calculation in backend/src/services/consolidation.rs (for name similarity matching)
- [X] T040 [US1] Implement conflict detection in backend/src/services/consolidation.rs (version mismatch, author mismatch, license conflict per provider-normalization.md test cases 6-7)
- [X] T041 [US1] Implement license compatibility matrix in backend/src/services/consolidation.rs (MIT+Apache=compatible, GPL-2.0+Apache=incompatible per provider-normalization.md)
- [X] T042 [US1] Create ConsolidatedModProfile builder in backend/src/services/consolidation.rs (aggregate metrics, detect conflicts, compute confidence score)

### Caching for User Story 1

- [X] T043 [P] [US1] Implement provider response cache in backend/src/services/cache/provider_cache.rs (SQLite storage in cache_db.sqlite with 24h TTL per research.md)
- [X] T044 [P] [US1] Implement consolidated result cache in backend/src/services/cache/result_cache.rs (cache_key: sha256(provider + query + filters), 24h TTL)
- [X] T045 [US1] Integrate provider cache into Modrinth adapter in backend/src/providers/modrinth.rs (check cache before API call, store on success)
- [X] T046 [US1] Integrate provider cache into CurseForge adapter in backend/src/providers/curseforge.rs (check cache before API call, store on success)

### REST API for User Story 1

- [X] T047 [P] [US1] Create axum router in backend/src/api/main.rs (mount routes: /api/v1/health, /api/v1/search per rest-api.md)
- [X] T048 [P] [US1] Implement POST /api/v1/search endpoint in backend/src/api/handlers/search.rs (accept SearchRequest, return SearchResponse per rest-api.md)
- [X] T049 [US1] Integrate MultiProviderOrchestrator into search handler in backend/src/api/handlers/search.rs (concurrent provider queries)
- [X] T050 [US1] Integrate ConsolidationService into search handler in backend/src/api/handlers/search.rs (merge and deduplicate results)
- [X] T051 [US1] Implement search response serialization in backend/src/api/handlers/search.rs (SearchResponse with data and metadata per rest-api.md)
- [X] T052 [US1] Add degraded mode notification in backend/src/api/handlers/search.rs (provider_errors field with severity and reason per research.md error aggregation)
- [X] T053 [P] [US1] Implement GET /api/v1/health endpoint in backend/src/api/handlers/health.rs (return status and version)
- [X] T054 [P] [US1] Add CORS middleware in backend/src/api/middleware/cors.rs (allow frontend origin)

### CLI Client for User Story 1

- [X] T055 [P] [US1] Create CLI argument parser in backend/src/cli/main.rs (clap with search subcommand, keywords, version, loader filters per quickstart.md)
- [X] T056 [US1] Implement CLI search command in backend/src/cli/commands/search.rs (call MultiProviderOrchestrator directly, reuse backend services)
- [X] T057 [US1] Implement CLI result formatter in backend/src/cli/formatters/mod.rs (colored output, show provider labels and normalized metadata)
- [X] T058 [US1] Implement CLI error display in backend/src/cli/formatters/error.rs (show degraded mode notices, provider failures)

### Web UI for User Story 1

- [X] T059 [P] [US1] Create SearchForm component in frontend/src/components/SearchForm.tsx (keyword input, provider checkboxes per rest-api.md)
- [X] T060 [P] [US1] Create ResultList component in frontend/src/components/ResultList.tsx (display consolidated results with provider labels)
- [X] T061 [P] [US1] Create ResultCard component in frontend/src/components/ResultCard.tsx (show mod name, summary, downloads, versions, loaders)
- [X] T062 [US1] Create API client service in frontend/src/services/api.ts (axios wrapper for POST /api/v1/search)
- [X] T063 [US1] Create SearchPage in frontend/src/pages/SearchPage.tsx (integrate SearchForm, ResultList, API client with react-query)
- [X] T064 [US1] Add degraded mode notice component in frontend/src/components/DegradedModeNotice.tsx (show provider errors, severity warnings)

### Client-Local Caching for User Story 1

- [X] T065 [P] [US1] Implement client-local session cache in frontend/src/services/sessionCache.ts (localStorage persistence with cached_at, expires_at, cache_version per research.md)
- [X] T066 [P] [US1] Implement CLI session cache in backend/src/cli/cache/session_cache.rs (SQLite storage in data/cli_sessions.db with freshness metadata)
- [X] T067 [US1] Add freshness check to Web UI cache in frontend/src/services/sessionCache.ts (validate cache_version and expires_at before reuse per NFR-007)
- [X] T068 [US1] Add freshness check to CLI cache in backend/src/cli/cache/session_cache.rs (validate expires_at, force refresh on version mismatch per NFR-007)
- [X] T069 [US1] Add explicit refresh control in frontend/src/components/SearchForm.tsx (refresh button to bypass cache)
- [X] T070 [US1] Add explicit refresh control in backend/src/cli/commands/search.rs (--force-refresh flag to bypass cache)

**Checkpoint**: User Story 1 complete - unified search works in CLI and Web UI with identical results

---

## Phase 4: User Story 2 - Advanced Version-Aware Filtering (Priority: P2)

**Goal**: Filter by compatibility metadata and sort by updates relevant to target Minecraft version

**Independent Test**: Run search constrained to legacy Minecraft version (1.7.10) with advanced filters; confirm only compatible items returned and ranked by version-relevant update recency

### Advanced Filtering for User Story 2

- [X] T071 [P] [US2] Implement Minecraft version filter in backend/src/services/filtering/version_filter.rs (semantic version matching, range expansion per provider-normalization.md)
- [X] T072 [P] [US2] Implement loader filter in backend/src/services/filtering/loader_filter.rs (match supported_loaders against requested loaders)
- [X] T073 [P] [US2] Implement category filter in backend/src/services/filtering/category_filter.rs (match mod categories against requested categories)
- [X] T074 [P] [US2] Implement update recency filter in backend/src/services/filtering/recency_filter.rs (filter by update_recency_window Duration)
- [X] T075 [P] [US2] Implement download threshold filter in backend/src/services/filtering/download_filter.rs (filter by min_downloads)
- [X] T076 [P] [US2] Implement open source filter in backend/src/services/filtering/license_filter.rs (filter by open_source_only flag)
- [X] T077 [US2] Create FilterService in backend/src/services/filtering/mod.rs (apply all filters to consolidated results, combine filter results with AND logic)

### Version-Aware Sorting for User Story 2

- [X] T078 [US2] Implement version-aware update tracking in backend/src/services/version_aware_ranking.rs (extract last_update_for_version from provider version lists per FR-005)
- [X] T079 [US2] Fetch version-specific update dates from Modrinth in backend/src/providers/modrinth.rs (call /projects/{id}/versions endpoint, filter by target MC version)
- [X] T080 [US2] Fetch version-specific update dates from CurseForge in backend/src/providers/curseforge.rs (parse /mods/{id}/files, filter by gameVersion matching target)
- [X] T081 [US2] Implement relevance sort in backend/src/services/sorting/relevance.rs (keyword match score + download popularity)
- [X] T082 [US2] Implement version-aware recency sort in backend/src/services/sorting/version_aware_recency.rs (rank by last_update_for_version instead of global last_updated per FR-005)
- [X] T083 [US2] Implement downloads sort in backend/src/services/sorting/downloads.rs (rank by total_downloads descending)
- [X] T084 [US2] Implement created date sort in backend/src/services/sorting/created_date.rs (rank by earliest_created descending)
- [X] T085 [US2] Create SortingService in backend/src/services/sorting/mod.rs (apply sort_mode to filtered results)

### API Integration for User Story 2

- [X] T086 [US2] Extend POST /api/v1/search request schema in backend/src/api/handlers/search.rs (add filters and sort_mode fields per rest-api.md)
- [X] T087 [US2] Integrate FilterService into search handler in backend/src/api/handlers/search.rs (apply filters after consolidation)
- [X] T088 [US2] Integrate SortingService into search handler in backend/src/api/handlers/search.rs (apply sorting after filtering)
- [X] T166 [US2] Implement search result pagination/limit controls in backend/src/api/handlers/search.rs and frontend/src/services/api.ts (prevent overwhelming result volume, include pagination metadata)

### CLI Integration for User Story 2

- [X] T089 [US2] Add filter flags to CLI in backend/src/cli/main.rs (--version, --loader, --category, --min-downloads, --open-source, --updated-within)
- [X] T090 [US2] Add sort flag to CLI in backend/src/cli/main.rs (--sort with values: relevance, update-recency, downloads, created)
- [X] T091 [US2] Integrate filters into CLI search command in backend/src/cli/commands/search.rs (build SearchFilters from CLI args, pass to search)
- [X] T092 [US2] Integrate sorting into CLI search command in backend/src/cli/commands/search.rs (build SortMode from CLI args, pass to search)

### Web UI Integration for User Story 2

- [X] T093 [P] [US2] Create FilterPanel component in frontend/src/components/FilterPanel.tsx (MC version dropdown, loader checkboxes, category multi-select, download threshold slider, open source toggle, recency window dropdown)
- [X] T094 [P] [US2] Create SortSelector component in frontend/src/components/SortSelector.tsx (dropdown with relevance, update-recency, downloads, created options)
- [X] T095 [US2] Integrate FilterPanel into SearchPage in frontend/src/pages/SearchPage.tsx (collect filter state, pass to API client)
- [X] T096 [US2] Integrate SortSelector into SearchPage in frontend/src/pages/SearchPage.tsx (collect sort state, pass to API client)
- [X] T097 [US2] Update API client in frontend/src/services/api.ts (include filters and sort_mode in POST /api/v1/search request body)

**Checkpoint**: User Story 2 complete - advanced filtering and version-aware sorting work in CLI and Web UI

---

## Phase 5: User Story 3 - Deep Discovery and Composite Insights (Priority: P3)

**Goal**: Deep discovery for relationships not in provider search + composite mod view for confident decision-making

**Independent Test**: Execute query requiring relationship discovery, wait for completion, verify results include discovered signals + integrated summary per mod

### Deep Discovery Implementation for User Story 3

- [X] T098 [P] [US3] Create DiscoveryService in backend/src/services/discovery/mod.rs (analyze relationships beyond provider search per FR-006)
- [X] T099 [P] [US3] Implement dependency detection in backend/src/services/discovery/dependency_detector.rs (parse mod metadata for required/optional dependencies)
- [X] T100 [P] [US3] Implement incompatibility detection in backend/src/services/discovery/incompatibility_detector.rs (identify known conflicts from provider metadata)
- [X] T101 [P] [US3] Implement replacement detection in backend/src/services/discovery/replacement_detector.rs (identify mod alternatives/successors)
- [X] T102 [P] [US3] Implement complement detection in backend/src/services/discovery/complement_detector.rs (identify mods that work well together)
- [X] T103 [US3] Create discovery evidence builder in backend/src/services/discovery/evidence_builder.rs (create DiscoveryEvidenceItem instances with confidence scores per data-model.md)
- [X] T104 [US3] Implement discovery cache in backend/src/services/cache/discovery_cache.rs (SQLite storage in discovery_cache.sqlite with 30-day TTL per research.md)

### Progress Tracking for User Story 3

- [X] T105 [US3] Create progress tracking service in backend/src/services/progress/mod.rs (track discovery state: pending, processing, complete per FR-007)
- [X] T106 [US3] Implement SSE endpoint in backend/src/api/handlers/progress.rs (GET /api/v1/search/{id}/progress for real-time updates to Web UI)
- [X] T107 [US3] Add progress state to SearchResponse in backend/src/api/handlers/search.rs (include discovery_state field: pending/processing/complete)
- [X] T108 [US3] Implement CLI progress bar in backend/src/cli/formatters/progress.rs (indicatif progress bar showing "Searching providers (✓), discovering relationships (12/47)...")
- [X] T109 [US3] Block final results until discovery complete in backend/src/services/orchestrator.rs (wait for DiscoveryService completion before returning results per FR-007)

### Composite Mod View for User Story 3

- [X] T110 [P] [US3] Create compatibility assessment service in backend/src/services/analysis/compatibility_assessor.rs (compute CompatibilityStatus: FullyCompatible/PartiallyCompatible/Incompatible per data-model.md)
- [X] T111 [P] [US3] Create maintenance signal analyzer in backend/src/services/analysis/maintenance_analyzer.rs (compute MaintenanceStatus: ActivelyMaintained/Maintenance/Abandoned per data-model.md)
- [X] T112 [P] [US3] Create risk factor detector in backend/src/services/analysis/risk_detector.rs (identify RiskFactor instances: LicenseConflict, DependencyIssue, ProviderDiscrepancy, OutdatedVersion, LowAdoption per data-model.md)
- [X] T113 [US3] Create composite profile builder in backend/src/services/analysis/composite_builder.rs (integrate compatibility, maintenance, risks, discovery evidence into ConsolidatedModProfile)
- [X] T114 [US3] Create summary generator in backend/src/services/analysis/summary_generator.rs (generate user-readable summary of compatibility, maintenance, risks per FR-009)

### API Endpoints for User Story 3

- [X] T115 [US3] Implement GET /api/v1/mods/{id} endpoint in backend/src/api/handlers/mod_details.rs (return ConsolidatedModProfile with composite view per rest-api.md)
- [X] T116 [US3] Add discovery evidence to mod details response in backend/src/api/handlers/mod_details.rs (include DiscoveryEvidenceItem array with relationships)
- [X] T117 [US3] Add metadata conflicts to mod details response in backend/src/api/handlers/mod_details.rs (show MetadataConflict array with provider discrepancies per FR-008)

### CLI for User Story 3

- [X] T118 [US3] Implement details subcommand in backend/src/cli/commands/details.rs (show composite profile for selected mod)
- [X] T119 [US3] Add discovery evidence display to CLI in backend/src/cli/formatters/details.rs (show relationships with confidence scores)
- [X] T120 [US3] Add metadata conflict display to CLI in backend/src/cli/formatters/details.rs (highlight conflicts with severity indicators)
- [X] T121 [US3] Add integrated summary display to CLI in backend/src/cli/formatters/details.rs (show compatibility, maintenance, risks in readable format)

### Web UI for User Story 3

- [X] T122 [P] [US3] Create ModDetailPage in frontend/src/pages/ModDetailPage.tsx (composite view with all evidence)
- [X] T123 [P] [US3] Create CompatibilitySection component in frontend/src/components/ModDetail/CompatibilitySection.tsx (show CompatibilityStatus with target version context)
- [X] T124 [P] [US3] Create MaintenanceSection component in frontend/src/components/ModDetail/MaintenanceSection.tsx (show MaintenanceStatus with update activity)
- [X] T125 [P] [US3] Create RiskFactorsSection component in frontend/src/components/ModDetail/RiskFactorsSection.tsx (list RiskFactor items with severity badges)
- [X] T126 [P] [US3] Create RelationshipsSection component in frontend/src/components/ModDetail/RelationshipsSection.tsx (show DiscoveryEvidenceItem with relationship types)
- [X] T127 [P] [US3] Create MetadataConflictsSection component in frontend/src/components/ModDetail/MetadataConflictsSection.tsx (highlight provider discrepancies with severity)
- [X] T128 [P] [US3] Create IntegratedSummary component in frontend/src/components/ModDetail/IntegratedSummary.tsx (display generated summary from summary_generator)
- [X] T129 [US3] Integrate all sections into ModDetailPage in frontend/src/pages/ModDetailPage.tsx (layout with compatibility, maintenance, risks, relationships, conflicts, summary)
- [X] T130 [US3] Add progress indicator to SearchPage in frontend/src/components/ProgressIndicator.tsx (connect to SSE endpoint, show discovery progress while processing)

### Session Preservation for User Story 3

- [X] T131 [P] [US3] Create session service in backend/src/services/session/mod.rs (create, update, retrieve SearchSessionSummary per data-model.md)
- [X] T132 [P] [US3] Implement session persistence in backend/src/services/session/persistence.rs (store in sessions_db.sqlite with 30-day TTL)
- [X] T133 [US3] Implement POST /api/v1/sessions endpoint in backend/src/api/handlers/sessions.rs (save shortlisted mods with comparison notes)
- [X] T134 [US3] Implement GET /api/v1/sessions/{id} endpoint in backend/src/api/handlers/sessions.rs (retrieve saved session)
- [X] T135 [US3] Implement GET /api/v1/sessions endpoint in backend/src/api/handlers/sessions.rs (list user's recent sessions)
- [X] T136 [US3] Add save session command to CLI in backend/src/cli/commands/save_session.rs (persist current search results with notes)
- [X] T137 [US3] Add load session command to CLI in backend/src/cli/commands/load_session.rs (retrieve and display saved session)
- [X] T138 [US3] Create SavedSessionsList component in frontend/src/components/SavedSessionsList.tsx (show recent sessions with timestamps)
- [X] T139 [US3] Create SessionComparison component in frontend/src/components/SessionComparison.tsx (compare multiple mods side-by-side per FR-012)
- [X] T140 [US3] Add save button to SearchPage in frontend/src/pages/SearchPage.tsx (save current shortlist with user notes)

**Checkpoint**: User Story 3 complete - deep discovery, composite insights, and session preservation work end-to-end

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories and production readiness

### Documentation

- [X] T141 [P] Update README.md with installation instructions, usage examples for CLI and Web UI, configuration guide
- [X] T142 [P] Create API documentation in backend/docs/api.md (OpenAPI spec generation from axum routes)
- [X] T143 [P] Create CLI help documentation in backend/docs/cli.md (usage examples, filter options, sort modes)
- [X] T144 [P] Create deployment guide in backend/docs/deployment.md (binary compilation, database setup, environment variables)
- [X] T167 [P] Validate and maintain acceptance-evaluation protocol in specs/001-multi-provider-mod-search/quickstart.md (sample size, task scenarios, rating method for NFR-004/NFR-005)

### Performance Optimization

- [X] T145 [P] Add connection pooling for provider HTTP clients in backend/src/providers/mod.rs (reuse reqwest::Client instances)
- [X] T146 [P] Optimize database query indexes in backend/migrations/ (add missing indexes from data-model.md)
- [X] T147 Run performance benchmarks from quickstart.md (cargo bench, verify NFR-001: 95% searches complete within 30s)
- [X] T148 Optimize cache hit rate (verify NFR-006: 70% repeated searches complete in 3s via cache)

### Security Hardening

- [X] T149 [P] Add input validation for all API endpoints in backend/src/api/validation.rs (sanitize keywords, validate version formats, limit filter array sizes)
- [X] T150 [P] Add rate limiting per IP in backend/src/api/middleware/rate_limiter.rs (per-IP throttling per research.md)
- [X] T151 [P] Secure CurseForge API key management in backend/src/config/secrets.rs (read from environment, never log)
- [X] T152 [P] Add HTTPS/TLS configuration in backend/src/api/main.rs (axum with rustls)

### Testing & Validation

- [X] T153 [P] Run all contract tests from quickstart.md (cargo test --test contract, verify provider normalization per provider-normalization.md test cases 1-10)
- [X] T154 [P] Run all integration tests from quickstart.md (cargo test --test integration, verify end-to-end search workflows)
- [X] T155 [P] Run quickstart.md validation checklist (verify all phases pass acceptance criteria)
- [X] T156 Run constitution compliance review (verify all FR requirements met, NFR targets validated)
- [X] T168 [US1] Add edge-case integration test in backend/tests/integration/provider_outage_and_stale_cache_test.rs (provider unavailable + stale client cache behavior per spec edge cases)
- [X] T169 [US2] Add edge-case integration test in backend/tests/integration/high_volume_and_rare_version_test.rs (broad result volume pagination + rare version filtering behavior)
- [X] T170 [US3] Add edge-case integration test in backend/tests/integration/ambiguous_relationship_confidence_test.rs (weak discovery signals and confidence handling)
- [X] T171 [P] Add ordering consistency integration test in backend/tests/integration/ordering_consistency_test.rs (repeat identical query and verify NFR-002 consistency target)
- [X] T172 [P] Add provider extensibility contract test in backend/tests/contract/provider_extensibility_test.rs (mock GitHub adapter integration without breaking required providers)

### Code Quality

- [X] T157 [P] Code cleanup and refactoring (remove unused imports, simplify complex functions)
- [X] T158 [P] Add comprehensive error messages (user-friendly error descriptions for all failure modes)
- [X] T159 [P] Add logging for all critical operations (search requests, provider failures, cache hits/misses, discovery completion)

### Deployment Preparation

- [X] T160 [P] Build release binaries for Linux in backend/ (cargo build --release --target x86_64-unknown-linux-gnu)
- [ ] T161 [P] Build release binaries for macOS in backend/ (cargo build --release --target x86_64-apple-darwin) — Blocked: Linux environment lacks macOS cross-compilation toolchain (`cc` does not support `-arch` / `-mmacosx-version-min`)
- [ ] T162 [P] Build release binaries for Windows in backend/ (cargo build --release --target x86_64-pc-windows-msvc) — Blocked: Linux environment lacks Windows MSVC-compatible cross-compilation toolchain
- [X] T163 [P] Build frontend production bundle in frontend/ (npm run build)
- [X] T164 Create Docker container for backend (Dockerfile with Rust binary and SQLite databases)
- [X] T165 Create Docker Compose configuration (backend API + frontend static hosting + database volumes)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User Story 1 (Phase 3): Can start after Foundational - No dependencies on other stories
  - User Story 2 (Phase 4): Can start after Foundational - Extends US1 but independently testable
  - User Story 3 (Phase 5): Can start after Foundational - Extends US1/US2 but independently testable
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 - No dependencies on other stories
  - Delivers: Multi-provider search with consolidation and caching
  - Independent test: Same query in CLI and Web UI returns identical results
  
- **User Story 2 (P2)**: Can start after Phase 2 - Integrates with US1 but independently testable
  - Delivers: Advanced filtering and version-aware sorting
  - Independent test: Legacy version search (1.7.10) returns only compatible mods ranked by version-relevant updates
  - Integration: Extends US1 search with FilterService and SortingService
  
- **User Story 3 (P3)**: Can start after Phase 2 - Integrates with US1/US2 but independently testable
  - Delivers: Deep discovery, composite insights, session preservation
  - Independent test: Query with discovery shows relationships and integrated summary
  - Integration: Adds DiscoveryService and composite view to US1 results, applies US2 filters

### Within Each User Story

**User Story 1**:
1. Provider adapters (T023-T030) can run in parallel [P]
2. Orchestrator (T031-T036) depends on adapters
3. Consolidation (T037-T042) depends on orchestrator
4. Caching (T043-T046) can run in parallel with consolidation [P]
5. API endpoints (T047-T054) can run after consolidation
6. CLI (T055-T058) can run in parallel with API [P]
7. Web UI (T059-T064) can run in parallel with API [P]
8. Client-local caching (T065-T070) extends CLI and Web UI

**User Story 2**:
1. Filtering (T071-T077) can run in parallel [P]
2. Version-aware ranking (T078-T085) can run in parallel with filtering [P]
3. API integration (T086-T088) depends on filtering and sorting
4. CLI integration (T089-T092) depends on filtering and sorting
5. Web UI integration (T093-T097) can run in parallel with CLI [P]

**User Story 3**:
1. Discovery (T098-T104) can run in parallel [P]
2. Progress tracking (T105-T109) can run in parallel with discovery [P]
3. Composite view (T110-T114) can run in parallel [P]
4. API endpoints (T115-T117) depend on composite view
5. CLI (T118-T121) can run in parallel with API [P]
6. Web UI (T122-T130) can run in parallel with API and CLI [P]
7. Session preservation (T131-T140) extends all interfaces

### Parallel Opportunities

**Setup Phase (T001-T006)**:
- T003, T004, T005, T006 can all run in parallel after T001-T002

**Foundational Phase (T007-T022)**:
- Database setup (T007-T009) can run in parallel
- All models (T010-T015) can run in parallel after T009
- Provider infrastructure (T016-T019) can run in parallel with models
- Error handling (T020-T022) can run in parallel

**User Story 1 (T023-T070)**:
- Provider adapters: T023-T030 all parallel (8 tasks)
- Caching: T043-T044 parallel (2 tasks)
- API endpoints: T047, T053, T054 parallel (3 tasks)
- Web UI components: T059-T061 parallel (3 tasks)
- Client caches: T065-T066 parallel (2 tasks)

**User Story 2 (T071-T097)**:
- All filters: T071-T076 parallel (6 tasks)
- All sorters: T081-T084 parallel (4 tasks)
- Web UI: T093-T094 parallel (2 tasks)

**User Story 3 (T098-T140)**:
- Discovery detectors: T099-T102 parallel (4 tasks)
- Analysis services: T110-T112 parallel (3 tasks)
- Web UI sections: T123-T128 parallel (6 tasks)
- Session services: T131-T132 parallel (2 tasks)

**Polish Phase (T141-T165)**:
- Documentation: T141-T144 parallel (4 tasks)
- Performance: T145-T146 parallel (2 tasks)
- Security: T149-T152 parallel (4 tasks)
- Testing: T153-T155 parallel (3 tasks)
- Cleanup: T157-T159 parallel (3 tasks)
- Binaries: T160-T163 parallel (4 tasks)

---

## Parallel Execution Example: User Story 1

```bash
# Launch all provider adapters in parallel:
Task T023: "Implement Modrinth adapter in backend/src/providers/modrinth.rs"
Task T024: "Implement CurseForge adapter in backend/src/providers/curseforge.rs"
Task T025: "Create Modrinth response structs in backend/src/providers/modrinth.rs"
Task T026: "Create CurseForge response structs in backend/src/providers/curseforge.rs"
Task T027: "Modrinth loader normalization in backend/src/providers/modrinth.rs"
Task T028: "CurseForge loader normalization in backend/src/providers/curseforge.rs"
Task T029: "Modrinth version parsing in backend/src/providers/modrinth.rs"
Task T030: "CurseForge version extraction in backend/src/providers/curseforge.rs"

# After orchestrator/consolidation, launch all interfaces in parallel:
Task T047: "Create axum router in backend/src/api/main.rs"
Task T055: "Create CLI argument parser in backend/src/cli/main.rs"
Task T059: "Create SearchForm component in frontend/src/components/SearchForm.tsx"
Task T060: "Create ResultList component in frontend/src/components/ResultList.tsx"
Task T061: "Create ResultCard component in frontend/src/components/ResultCard.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T022) ← CRITICAL BLOCKER
3. Complete Phase 3: User Story 1 (T023-T070)
4. **STOP and VALIDATE**: Test unified search in CLI and Web UI independently
5. Deploy/demo if ready

**MVP Deliverable**: Multi-provider search with Modrinth and CurseForge consolidation, available via CLI and Web UI with caching

### Incremental Delivery

1. Complete Setup + Foundational (Phases 1-2) → Foundation ready
2. Add User Story 1 (Phase 3) → Test independently → **Deploy/Demo MVP**
3. Add User Story 2 (Phase 4) → Test independently → Deploy/Demo (advanced filtering)
4. Add User Story 3 (Phase 5) → Test independently → Deploy/Demo (deep discovery)
5. Polish (Phase 6) → Production-ready

Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (Phases 1-2)
2. Once Foundational is done:
   - Developer A: User Story 1 (T023-T070)
   - Developer B: User Story 2 (T071-T097) - can start early on filtering/sorting services
   - Developer C: User Story 3 (T098-T140) - can start early on discovery services
3. Stories integrate independently into shared backend services

---

## Notes

- **[P] tasks** = different files, no dependencies within phase
- **[Story] label** maps task to specific user story for traceability
- Each user story is independently completable and testable
- Cache enhancements: Multi-layer (provider 24h + consolidated query + client-local session with freshness checks)
- Client-local persistence: Web UI uses localStorage, CLI uses SQLite with cache_version and expires_at validation
- Phased expansion note: GitHub and mc百科 are post-MVP providers and validated via extensibility tests before activation
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Total tasks: **172** (6 setup + 16 foundational + 48 US1 + 28 US2 + 43 US3 + 31 polish)
- Parallel opportunities: **80+ tasks** can run in parallel across different files
