# Data Model: Professional Multi-Provider Mod Finder

**Feature**: 001-multi-provider-mod-search  
**Date**: 2026-02-13  
**Phase**: Phase 1 - Design

## Overview

This document defines the core data entities, their relationships, validation rules, and state transitions for the MCM-Finder tool. All entities align with the Key Entities section from spec.md.

## Core Entities

### 1. SearchQueryProfile

**Purpose**: Captures user search intent with filters, provider scope, and sort preferences.

**Fields:**
```rust
struct SearchQueryProfile {
    // Identity
    id: Uuid,                          // Unique query identifier
    created_at: DateTime<Utc>,         // Query timestamp
    
    // Search parameters
    keywords: Vec<String>,             // Search terms (tokenized)
    filters: SearchFilters,            // Advanced filters
    provider_scope: Vec<Provider>,     // Enabled providers
    sort_mode: SortMode,               // Result ordering
    
    // Session tracking
    session_id: Option<Uuid>,          // Associated session (for preservation)
    user_context: Option<String>,      // IP/anonymous identifier for cache
}

struct SearchFilters {
    minecraft_version: Option<String>,      // Target MC version (e.g., "1.20.1")
    loaders: Vec<ModLoader>,                // Fabric, Forge, Quilt, NeoForge
    categories: Vec<String>,                // Mod categories/tags
    update_recency_window: Option<Duration>, // e.g., "updated within 6 months"
    min_downloads: Option<u64>,             // Download threshold
    open_source_only: bool,                 // License filter
}

enum SortMode {
    Relevance,                             // Default: keyword match + popularity
    UpdateRecencyVersionAware,             // Updates relevant to target MC version
    Downloads,                             // Total download count
    CreatedDate,                           // Newest mods first
}

enum ModLoader {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}

enum Provider {
    Modrinth,
    CurseForge,
    GitHub,      // Future
    McBaike,     // Future (mc百科)
}
```

**Validation Rules:**
- At least one keyword OR one filter must be present (non-empty query)
- `minecraft_version` must match semantic version format (e.g., "1.20.1", "1.7.10")
- `provider_scope` must contain at least one required provider (Modrinth or CurseForge)
- `update_recency_window` must be positive duration if specified

**Relationships:**
- One `SearchQueryProfile` → Many `ProviderResultRecord` (search results)
- One `SearchQueryProfile` → One `SearchSessionSummary` (optional, if preserved)

---

### 2. ProviderResultRecord

**Purpose**: Normalized representation of one provider's evidence for a mod candidate.

**Fields:**
```rust
struct ProviderResultRecord {
    // Identity
    id: Uuid,                          // Unique result identifier
    query_id: Uuid,                    // Parent query reference
    
    // Provider metadata
    source: Provider,                  // Origin provider
    provider_mod_id: String,           // Provider-specific ID (e.g., "AANobbMI")
    provider_url: String,              // Direct link to mod page
    fetched_at: DateTime<Utc>,         // Response timestamp
    
    // Normalized mod data
    name: String,                      // Mod name
    slug: String,                      // URL-safe identifier
    author: String,                    // Primary author/team
    summary: String,                   // Short description (<200 chars)
    description_html: Option<String>,  // Full description (HTML)
    
    // Compatibility
    supported_versions: Vec<String>,   // Minecraft versions
    supported_loaders: Vec<ModLoader>, // Mod loaders
    
    // Metrics
    downloads: u64,                    // Total download count
    followers: Option<u64>,            // Followers/subscribers
    last_updated: DateTime<Utc>,       // Most recent version release
    created_at: DateTime<Utc>,         // Project creation date
    
    // Metadata
    categories: Vec<String>,           // Tags/categories
    license: Option<String>,           // License type (e.g., "MIT", "GPL-3.0")
    source_url: Option<String>,        // Source code repository
    
    // Ranking
    relevance_score: f64,              // Provider's relevance score (0.0-1.0)
    
    // Version-aware update tracking (FR-005)
    last_update_for_version: Option<DateTime<Utc>>, // Last update for target MC version
}
```

**Validation Rules:**
- `name` must be non-empty and ≤ 100 characters
- `summary` must be ≤ 500 characters
- `supported_versions` must contain valid semantic versions
- `relevance_score` must be in range [0.0, 1.0]
- `downloads` and `followers` must be non-negative
- `fetched_at` must be ≤ current time

**Relationships:**
- Many `ProviderResultRecord` → One `SearchQueryProfile` (parent query)
- Many `ProviderResultRecord` → One `ConsolidatedModProfile` (via deduplication)

---

### 3. ConsolidatedModProfile

**Purpose**: Aggregated cross-provider view of one mod with compatibility signals and risk assessment.

**Fields:**
```rust
struct ConsolidatedModProfile {
    // Identity
    id: Uuid,                              // Unique consolidated profile
    canonical_name: String,                // Normalized name across providers
    canonical_slug: String,                // URL-safe identifier
    
    // Source aggregation
    provider_records: Vec<ProviderResultRecord>, // All provider results for this mod
    primary_source: Provider,              // Preferred/authoritative source
    
    // Consolidated metadata
    authors: Vec<String>,                  // All identified authors
    descriptions: HashMap<Provider, String>, // Per-provider descriptions
    
    // Compatibility (union of all providers)
    all_supported_versions: Vec<String>,   // All MC versions (any provider)
    all_supported_loaders: Vec<ModLoader>, // All loaders (any provider)
    
    // Aggregated metrics
    total_downloads: u64,                  // Sum across providers (deduplicated if possible)
    max_followers: u64,                    // Maximum followers from any provider
    earliest_created: DateTime<Utc>,       // Oldest creation date
    most_recent_update: DateTime<Utc>,     // Latest update across providers
    
    // Conflict detection (FR-008)
    metadata_conflicts: Vec<MetadataConflict>,
    
    // Discovery integration
    discovery_evidence: Vec<DiscoveryEvidenceItem>, // Relationship data
    
    // Decision support (FR-009)
    compatibility_assessment: CompatibilityStatus,
    maintenance_signals: MaintenanceStatus,
    adoption_risks: Vec<RiskFactor>,
    
    // Computed scores
    composite_relevance: f64,              // Aggregated relevance (0.0-1.0)
    confidence_score: f64,                 // Data quality/consistency (0.0-1.0)
}

struct MetadataConflict {
    field: String,                         // Conflicting field name
    values: HashMap<Provider, String>,     // Provider-specific values
    severity: ConflictSeverity,            // Minor | Major | Critical
}

enum ConflictSeverity {
    Minor,      // e.g., description wording differences
    Major,      // e.g., different version numbers
    Critical,   // e.g., conflicting license information
}

enum CompatibilityStatus {
    FullyCompatible,                       // Matches all target requirements
    PartiallyCompatible { issues: Vec<String> }, // Some filters not met
    Incompatible { reasons: Vec<String> }, // Does not match target
}

enum MaintenanceStatus {
    ActivelyMaintained,                    // Updated within 6 months for target version
    Maintenance { last_update_days: u32 }, // Updated but not recently
    Abandoned,                             // No updates for 2+ years
    Unknown,                               // Insufficient data
}

struct RiskFactor {
    risk_type: RiskType,
    description: String,
    severity: RiskSeverity,
}

enum RiskType {
    LicenseConflict,                       // Incompatible license
    DependencyIssue,                       // Missing or conflicting dependencies
    ProviderDiscrepancy,                   // Major differences between providers
    OutdatedVersion,                       // No recent updates for target MC version
    LowAdoption,                           // Few downloads/followers
}

enum RiskSeverity {
    Low,
    Medium,
    High,
}
```

**Validation Rules:**
- `provider_records` must contain at least 1 record
- `canonical_name` derived from majority provider name or highest-confidence source
- `metadata_conflicts` auto-populated during consolidation
- `confidence_score` decreases with number of unresolved conflicts
- `compatibility_assessment` must align with `SearchFilters` from originating query

**State Transitions:**
1. **Initialized**: Created from first `ProviderResultRecord`
2. **Aggregating**: Additional provider records merged
3. **Discovery Pending**: Awaiting deep discovery completion
4. **Complete**: All discovery evidence integrated, ready for presentation

**Relationships:**
- One `ConsolidatedModProfile` → Many `ProviderResultRecord` (sources)
- One `ConsolidatedModProfile` → Many `DiscoveryEvidenceItem` (relationships)
- Many `ConsolidatedModProfile` → One `SearchSessionSummary` (shortlist)

---

### 4. DiscoveryEvidenceItem

**Purpose**: Relationship evidence found through deep discovery beyond basic search output (FR-006).

**Fields:**
```rust
struct DiscoveryEvidenceItem {
    // Identity
    id: Uuid,
    mod_profile_id: Uuid,              // Associated consolidated profile
    
    // Relationship data
    relationship_type: RelationshipType,
    related_mod_id: Option<Uuid>,      // Target mod (if identified)
    related_mod_name: String,          // Name of related mod
    
    // Evidence
    evidence_source: EvidenceSource,
    confidence: f64,                   // Strength of relationship (0.0-1.0)
    discovered_at: DateTime<Utc>,
    
    // Context
    context: String,                   // Human-readable explanation
    metadata: serde_json::Value,       // Structured evidence data
}

enum RelationshipType {
    Dependency,                        // Required dependency
    OptionalDependency,                // Suggested/compatible dependency
    Incompatibility,                   // Known conflict
    Replacement,                       // Alternative/replacement for
    Complement,                        // Works well with
    Successor,                         // Newer version/fork of
}

enum EvidenceSource {
    ProviderApi,                       // From provider metadata
    SourceCodeAnalysis,                // Parsed from mod files/repo
    CommunityData,                     // Wiki, forum mentions
    VersionCompatibility,              // Inferred from version overlaps
}
```

**Validation Rules:**
- `confidence` must be in range [0.0, 1.0]
- `relationship_type` must be bidirectional consistent (A depends on B ⇔ B is dependency of A)
- Low-confidence evidence (< 0.5) flagged for user review

**Relationships:**
- Many `DiscoveryEvidenceItem` → One `ConsolidatedModProfile` (parent mod)
- One `DiscoveryEvidenceItem` → One `ConsolidatedModProfile` (related mod, optional)

---

### 5. SearchSessionSummary

**Purpose**: Preserved snapshot of query context, resulting shortlist, and comparison notes for team review (FR-011, FR-012).

**Fields:**
```rust
struct SearchSessionSummary {
    // Identity
    id: Uuid,                          // Session identifier
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>, // Auto-cleanup (default: 30 days)
    
    // Query context
    original_query: SearchQueryProfile,
    query_description: Option<String>, // User-provided label
    
    // Results
    shortlisted_mods: Vec<Uuid>,       // ConsolidatedModProfile IDs
    comparison_notes: HashMap<Uuid, String>, // Per-mod notes
    
    // Metadata
    search_duration_ms: u64,           // Total search time
    providers_used: Vec<Provider>,     // Active providers during search
    total_candidates_found: u32,       // Pre-filter result count
    
    // Sharing
    shareable_link: Option<String>,    // Public link for team review
    user_identifier: Option<String>,   // Anonymous user ID
}
```

**Validation Rules:**
- `created_at` ≤ `updated_at`
- `shortlisted_mods` must reference existing `ConsolidatedModProfile` records
- `expires_at` must be > `created_at` if present

**State Transitions:**
1. **Active**: Actively being modified
2. **Archived**: Read-only, preserved for review
3. **Expired**: Marked for deletion (TTL exceeded)

**Relationships:**
- One `SearchSessionSummary` → One `SearchQueryProfile` (original query)
- One `SearchSessionSummary` → Many `ConsolidatedModProfile` (shortlisted mods)

---

## Entity Relationship Diagram

```
SearchQueryProfile (1) ──> (N) ProviderResultRecord
                    └──> (1) SearchSessionSummary
                    
ProviderResultRecord (N) ──> (1) ConsolidatedModProfile

ConsolidatedModProfile (1) ──> (N) DiscoveryEvidenceItem
                       └──> (1) SearchSessionSummary (via shortlist)

DiscoveryEvidenceItem (N) ──> (1) ConsolidatedModProfile (optional related mod)

SearchSessionSummary (1) ──> (N) ConsolidatedModProfile (shortlisted)
```

---

## Indexes & Performance

**Database Indexes (SQLite):**
```sql
-- Searches
CREATE INDEX idx_query_hash ON searches(query_hash);
CREATE INDEX idx_session_user ON searches(user_identifier, created_at);

-- Provider cache
CREATE INDEX idx_provider_cache ON provider_responses(provider, cache_key, expires_at);

-- Discovery relationships
CREATE INDEX idx_relationships_mod ON relationships(mod_id_a, relationship_type);
CREATE INDEX idx_relationships_confidence ON relationships(confidence DESC);
```

**In-Memory Caching:**
- Active search sessions cached in memory (HashMap<Uuid, SearchSessionSummary>)
- Frequently accessed profiles cached with 5-min TTL
- Provider responses cached with 24h TTL

---

## Data Flow

1. **Search Initiation**: User creates `SearchQueryProfile` via CLI/Web UI
2. **Provider Aggregation**: System fetches `ProviderResultRecord` from each provider concurrently
3. **Normalization**: Results deduplicated and merged into `ConsolidatedModProfile`
4. **Deep Discovery**: System generates `DiscoveryEvidenceItem` for each profile
5. **Presentation**: Consolidated profiles with discovery evidence shown to user
6. **Session Preservation**: User saves shortlist as `SearchSessionSummary`

---

## Validation Summary

| Entity | Primary Validations |
|--------|-------------------|
| SearchQueryProfile | Non-empty query, valid version format, at least one required provider |
| ProviderResultRecord | Valid timestamps, score range [0.0, 1.0], non-negative metrics |
| ConsolidatedModProfile | At least one provider record, auto-conflict detection, compatibility alignment |
| DiscoveryEvidenceItem | Confidence range [0.0, 1.0], bidirectional consistency |
| SearchSessionSummary | Valid time ordering, existing profile references |

All validations enforce **Correctness Before Convenience** (Constitution Principle I).
