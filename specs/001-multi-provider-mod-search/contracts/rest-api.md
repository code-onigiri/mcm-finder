# REST API Contract: Professional Multi-Provider Mod Finder

**Feature**: 001-multi-provider-mod-search  
**Date**: 2026-02-13  
**Status**: Implementation-Ready  
**Base URL**: `http://localhost:3000/api/v1`

---

## Overview

This REST API contract defines the interface for the MCM-Finder system, enabling:
- Multi-provider unified search (Modrinth, CurseForge)
- Version-aware advanced filtering (FR-002, FR-004, FR-005)
- Deep discovery and progress tracking (FR-006, FR-007)
- Composite mod details with metadata conflict resolution (FR-008)
- Session preservation for team review (FR-011, FR-012)
- System health monitoring

**API Version**: 1.0  
**Authentication**: Optional (anonymous or user-scoped caching)  
**Rate Limiting**: 30 requests/minute per IP (see Rate Limiting section)

---

## Core Endpoints

### 1. POST /api/v1/search

**Summary**: Execute a multi-provider search with optional deep discovery.

**References**: FR-001, FR-002, FR-006, FR-007

**Request**:
```json
{
  "keywords": ["fabric", "performance"],
  "filters": {
    "minecraft_version": "1.20.1",
    "loaders": ["Fabric"],
    "categories": ["optimization"],
    "update_recency_window_days": 180,
    "min_downloads": 1000,
    "open_source_only": false
  },
  "provider_scope": ["Modrinth", "CurseForge"],
  "sort_mode": "UpdateRecencyVersionAware",
  "enable_deep_discovery": true,
  "discovery_timeout_seconds": 30
}
```

**Request Fields**:

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-----------|-------------|
| `keywords` | `string[]` | Yes | 1-10 terms, ≤50 chars each | Search terms (tokenized by system) |
| `filters` | `object` | No | See SearchFilters | Advanced filter set |
| `filters.minecraft_version` | `string` | No | Semantic version (e.g., "1.20.1") | Target Minecraft version |
| `filters.loaders` | `string[]` | No | Fabric, Forge, Quilt, NeoForge | Supported mod loaders |
| `filters.categories` | `string[]` | No | ≤20 categories | Mod categories/tags |
| `filters.update_recency_window_days` | `integer` | No | Positive integer | Only mods updated within N days |
| `filters.min_downloads` | `integer` | No | Non-negative | Minimum download threshold |
| `filters.open_source_only` | `boolean` | No | Default: false | Exclude proprietary licenses |
| `provider_scope` | `string[]` | Yes | Min 1, max 4 | Providers to search: Modrinth, CurseForge, (GitHub, McBaike future) |
| `sort_mode` | `string` | No | Default: Relevance | Relevance, UpdateRecencyVersionAware, Downloads, CreatedDate |
| `enable_deep_discovery` | `boolean` | No | Default: true | Perform relationship discovery |
| `discovery_timeout_seconds` | `integer` | No | Default: 30 | Max time to wait for discovery |

**Response** (202 Accepted with Location header):
```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "queued",
  "estimated_completion_ms": 15000,
  "_links": {
    "progress": "/api/v1/search/550e8400-e29b-41d4-a716-446655440000/discovery",
    "results": "/api/v1/search/550e8400-e29b-41d4-a716-446655440000/results"
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `query_id` | `uuid` | Unique search identifier for polling progress |
| `status` | `string` | Status: queued, searching, discovering, complete, failed |
| `estimated_completion_ms` | `integer` | Estimated milliseconds until results ready |
| `_links.progress` | `string` | Link to poll discovery progress |
| `_links.results` | `string` | Link to retrieve consolidated results (once complete) |

**Status Codes**:
- **202 Accepted**: Query queued successfully
- **400 Bad Request**: Invalid query parameters (missing keywords, invalid version format, etc.)
- **429 Too Many Requests**: Rate limit exceeded
- **503 Service Unavailable**: All providers currently unavailable

**Example Error Response** (400):
```json
{
  "error": {
    "code": "INVALID_QUERY",
    "message": "Keywords required and must contain 1-10 terms",
    "details": {
      "field": "keywords",
      "constraint": "array_length_bounds",
      "min": 1,
      "max": 10
    }
  }
}
```

**Notes**:
- This endpoint **never blocks** on deep discovery. Results are shown via polling.
- If `enable_deep_discovery=false`, partial results returned immediately (~2-5 seconds).
- Search results are **not** published until discovery completes (FR-007).

---

### 2. GET /api/v1/search/{query_id}/discovery

**Summary**: Poll search progress and streaming discovery status.

**References**: FR-006, FR-007

**Request Parameters**:
```
GET /api/v1/search/550e8400-e29b-41d4-a716-446655440000/discovery
```

**Response** (200 OK):
```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "discovering",
  "progress": {
    "phase": "deep_discovery",
    "percentage": 65,
    "current_stage": "analyzing_relationships",
    "mods_processed": 23,
    "total_mods": 35,
    "elapsed_ms": 8500,
    "estimated_remaining_ms": 4500
  },
  "provider_status": {
    "Modrinth": {
      "status": "complete",
      "results_count": 45,
      "fetched_at": "2026-02-13T10:45:32.123Z"
    },
    "CurseForge": {
      "status": "complete",
      "results_count": 38,
      "fetched_at": "2026-02-13T10:45:35.456Z"
    }
  },
  "discovered_relationships_count": 12,
  "_links": {
    "results": "/api/v1/search/550e8400-e29b-41d4-a716-446655440000/results"
  }
}
```

**Status Values**:
- `queued`: Awaiting provider responses
- `searching`: Fetching from providers
- `discovering`: Processing relationships and conflicts
- `complete`: Results ready
- `failed`: Search failed (see `error` field)
- `partial`: One provider failed, partial results available

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `query_id` | `uuid` | Search identifier |
| `status` | `string` | Current phase (see Status Values) |
| `progress.phase` | `string` | Detailed phase: provider_fetch, aggregation, deep_discovery, conflict_resolution |
| `progress.percentage` | `integer` | Completion %: 0-100 |
| `progress.current_stage` | `string` | Human-readable stage description |
| `progress.mods_processed` | `integer` | Discovery items analyzed |
| `progress.total_mods` | `integer` | Total mods requiring analysis |
| `progress.elapsed_ms` | `integer` | Time elapsed since query start |
| `progress.estimated_remaining_ms` | `integer` | Estimated time to completion |
| `provider_status.<Provider>.status` | `string` | Per-provider status: pending, in_progress, complete, failed, timeout |
| `provider_status.<Provider>.results_count` | `integer` | Results from this provider |
| `provider_status.<Provider>.fetched_at` | `datetime` | Timestamp of fetch completion |
| `discovered_relationships_count` | `integer` | Relationships discovered so far |

**Degraded Coverage Scenario** (Partial Response):
```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "partial",
  "degraded_notice": {
    "unavailable_providers": ["CurseForge"],
    "reason": "Service temporarily unavailable",
    "message": "CurseForge is unavailable. Results from Modrinth only. Coverage: 1/2 providers.",
    "severity": "warning",
    "time_to_retry_seconds": 60
  },
  "provider_status": {
    "Modrinth": {
      "status": "complete",
      "results_count": 45
    },
    "CurseForge": {
      "status": "failed",
      "error": "timeout",
      "timeout_seconds": 10
    }
  },
  "progress": {
    "phase": "discovery_continuing",
    "percentage": 55
  }
}
```

**Status Codes**:
- **200 OK**: Progress returned successfully
- **202 Accepted**: Still processing (call again in 2-3 seconds)
- **404 Not Found**: Query ID invalid or expired
- **410 Gone**: Query completed and archived (>24 hours)

---

### 3. GET /api/v1/search/{query_id}/results

**Summary**: Retrieve consolidated search results (only available when discovery complete).

**References**: FR-001, FR-002, FR-008, FR-012

**Request Parameters**:
```
GET /api/v1/search/550e8400-e29b-41d4-a716-446655440000/results?offset=0&limit=20
```

**Query Parameters**:
- `offset` (integer, default: 0): Pagination offset
- `limit` (integer, default: 20, max: 100): Results per page

**Response** (200 OK):
```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "complete",
  "pagination": {
    "offset": 0,
    "limit": 20,
    "total": 83,
    "has_more": true
  },
  "search_summary": {
    "keywords": ["fabric", "performance"],
    "filters_applied": {
      "minecraft_version": "1.20.1",
      "loaders": ["Fabric"],
      "min_downloads": 1000
    },
    "total_duration_ms": 12453,
    "providers_used": ["Modrinth", "CurseForge"],
    "pre_filter_results": 127,
    "post_filter_results": 83
  },
  "consolidated_mods": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "canonical_name": "Sodium",
      "canonical_slug": "sodium",
      "authors": ["CaffeineMC"],
      "composite_relevance": 0.98,
      "confidence_score": 0.95,
      "compatibility_assessment": "FullyCompatible",
      "maintenance_signals": {
        "status": "ActivelyMaintained",
        "last_update_days": 3
      },
      "adoption_risks": [],
      "provider_records": [
        {
          "source": "Modrinth",
          "provider_mod_id": "AANobbMI",
          "name": "Sodium",
          "summary": "Modern rendering engine and client-side optimization mod for Minecraft",
          "provider_url": "https://modrinth.com/mod/sodium",
          "downloads": 24500000,
          "followers": 15000,
          "last_updated": "2026-02-10T14:22:00Z",
          "supported_versions": ["1.20.1", "1.20", "1.19.2", "1.18.2"],
          "supported_loaders": ["Fabric"],
          "categories": ["optimization", "rendering"],
          "license": "LGPL-3.0",
          "source_url": "https://github.com/CaffeineMC/sodium-fabric",
          "relevance_score": 0.99
        },
        {
          "source": "CurseForge",
          "provider_mod_id": "394468",
          "name": "Sodium",
          "summary": "Modern rendering engine for Minecraft (client-side)",
          "provider_url": "https://www.curseforge.com/minecraft/mods/sodium",
          "downloads": 22100000,
          "followers": 12500,
          "last_updated": "2026-02-09T10:00:00Z",
          "supported_versions": ["1.20.1", "1.20", "1.19.2"],
          "supported_loaders": ["Fabric"],
          "categories": ["optimization"],
          "license": "LGPL-3.0",
          "source_url": null,
          "relevance_score": 0.97
        }
      ],
      "metadata_conflicts": [
        {
          "field": "last_updated",
          "severity": "Minor",
          "values": {
            "Modrinth": "2026-02-10T14:22:00Z",
            "CurseForge": "2026-02-09T10:00:00Z"
          },
          "resolution": "Modrinth value used (newer)"
        }
      ],
      "discovery_evidence": [
        {
          "relationship_type": "Dependency",
          "related_mod_name": "Fabric API",
          "evidence_source": "ProviderApi",
          "confidence": 0.99,
          "context": "Required by Sodium for Fabric support"
        },
        {
          "relationship_type": "Incompatibility",
          "related_mod_name": "OptiFine",
          "evidence_source": "VersionCompatibility",
          "confidence": 0.85,
          "context": "Known conflict with OptiFine rendering changes"
        },
        {
          "relationship_type": "Complement",
          "related_mod_name": "Lithium",
          "evidence_source": "CommunityData",
          "confidence": 0.92,
          "context": "Popular pairing with Lithium for additional optimization"
        }
      ],
      "all_supported_versions": ["1.20.1", "1.20", "1.19.2", "1.18.2"],
      "all_supported_loaders": ["Fabric"],
      "total_downloads": 46600000,
      "earliest_created": "2020-08-15T00:00:00Z",
      "most_recent_update": "2026-02-10T14:22:00Z",
      "primary_source": "Modrinth"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "canonical_name": "Lithium",
      "canonical_slug": "lithium",
      "authors": ["CaffeineMC"],
      "composite_relevance": 0.94,
      "confidence_score": 0.93,
      "compatibility_assessment": "FullyCompatible",
      "maintenance_signals": {
        "status": "ActivelyMaintained",
        "last_update_days": 7
      },
      "adoption_risks": [],
      "provider_records": [
        {
          "source": "Modrinth",
          "provider_mod_id": "gvQqBUqZ",
          "name": "Lithium",
          "summary": "No-compromises game logic performance mod",
          "downloads": 18200000,
          "supported_versions": ["1.20.1", "1.20"],
          "supported_loaders": ["Fabric"]
        }
      ],
      "metadata_conflicts": [],
      "discovery_evidence": [
        {
          "relationship_type": "Complement",
          "related_mod_name": "Sodium",
          "confidence": 0.92
        }
      ]
    }
  ],
  "degraded_notice": null,
  "_links": {
    "self": "/api/v1/search/550e8400-e29b-41d4-a716-446655440000/results?offset=0&limit=20",
    "next": "/api/v1/search/550e8400-e29b-41d4-a716-446655440000/results?offset=20&limit=20",
    "save_session": "/api/v1/sessions"
  }
}
```

**ConsolidatedMod Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `uuid` | Unique consolidated profile ID |
| `canonical_name` | `string` | Normalized mod name across providers |
| `canonical_slug` | `string` | URL-safe identifier |
| `authors` | `string[]` | All identified authors |
| `composite_relevance` | `number` (0.0-1.0) | Aggregated relevance score |
| `confidence_score` | `number` (0.0-1.0) | Data quality/consistency |
| `compatibility_assessment` | `enum` | FullyCompatible, PartiallyCompatible, Incompatible |
| `maintenance_signals` | `object` | Status and recency info |
| `adoption_risks` | `array` | Risk factors (see Risk Factors section) |
| `provider_records` | `array` | Original provider results |
| `metadata_conflicts` | `array` | Conflicting fields between providers |
| `discovery_evidence` | `array` | Relationships found (see Discovery Evidence) |
| `all_supported_versions` | `string[]` | Union of all MC versions |
| `all_supported_loaders` | `string[]` | Union of all loaders |
| `total_downloads` | `integer` | Aggregated download count |
| `primary_source` | `string` | Authoritative provider (Modrinth preferred) |

**Discovery Evidence Fields**:

| Field | Type | Values |
|-------|------|--------|
| `relationship_type` | `enum` | Dependency, OptionalDependency, Incompatibility, Replacement, Complement, Successor |
| `related_mod_name` | `string` | Name of related mod |
| `evidence_source` | `enum` | ProviderApi, SourceCodeAnalysis, CommunityData, VersionCompatibility |
| `confidence` | `number` (0.0-1.0) | Relationship confidence strength |
| `context` | `string` | Human-readable explanation |

**Status Codes**:
- **200 OK**: Results available
- **202 Accepted**: Still processing (poll `/discovery` endpoint)
- **404 Not Found**: Query ID invalid or expired
- **503 Service Unavailable**: Partial results (see `degraded_notice`)

---

### 4. GET /api/v1/mods/{id}

**Summary**: Retrieve comprehensive details for a single consolidated mod profile.

**References**: FR-003, FR-008, FR-009

**Request Parameters**:
```
GET /api/v1/mods/660e8400-e29b-41d4-a716-446655440001
```

**Response** (200 OK):
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "canonical_name": "Sodium",
  "canonical_slug": "sodium",
  "authors": ["CaffeineMC"],
  "descriptions": {
    "Modrinth": "Modern rendering engine and client-side optimization mod for Minecraft",
    "CurseForge": "Modern rendering engine for Minecraft (client-side)"
  },
  "full_description_html": "<h2>Sodium</h2><p>A modern rendering engine and client-side optimization mod...</p>",
  "composite_relevance": 0.98,
  "confidence_score": 0.95,
  "primary_source": "Modrinth",
  "provider_records": [
    {
      "source": "Modrinth",
      "provider_mod_id": "AANobbMI",
      "provider_url": "https://modrinth.com/mod/sodium",
      "name": "Sodium",
      "summary": "Modern rendering engine and client-side optimization mod for Minecraft",
      "description_html": "<h2>Sodium</h2><p>...</p>",
      "downloads": 24500000,
      "followers": 15000,
      "last_updated": "2026-02-10T14:22:00Z",
      "created_at": "2020-08-15T00:00:00Z",
      "supported_versions": ["1.20.1", "1.20", "1.19.2", "1.18.2", "1.17.1"],
      "supported_loaders": ["Fabric"],
      "categories": ["optimization", "rendering"],
      "license": "LGPL-3.0",
      "source_url": "https://github.com/CaffeineMC/sodium-fabric",
      "relevance_score": 0.99
    },
    {
      "source": "CurseForge",
      "provider_mod_id": "394468",
      "provider_url": "https://www.curseforge.com/minecraft/mods/sodium",
      "name": "Sodium",
      "summary": "Modern rendering engine for Minecraft (client-side)",
      "downloads": 22100000,
      "followers": 12500,
      "last_updated": "2026-02-09T10:00:00Z",
      "created_at": "2020-08-17T00:00:00Z",
      "supported_versions": ["1.20.1", "1.20", "1.19.2"],
      "supported_loaders": ["Fabric"],
      "categories": ["optimization"],
      "license": "LGPL-3.0"
    }
  ],
  "metadata_conflicts": [
    {
      "field": "created_at",
      "severity": "Minor",
      "values": {
        "Modrinth": "2020-08-15",
        "CurseForge": "2020-08-17"
      },
      "resolution": "Modrinth value used (earlier)"
    },
    {
      "field": "supported_versions",
      "severity": "Minor",
      "values": {
        "Modrinth": "1.20.1, 1.20, 1.19.2, 1.18.2, 1.17.1",
        "CurseForge": "1.20.1, 1.20, 1.19.2"
      },
      "resolution": "Union used (more permissive)"
    }
  ],
  "compatibility_assessment": {
    "status": "FullyCompatible",
    "target_version": "1.20.1",
    "assessment": "Mod supports target Minecraft version 1.20.1 with Fabric loader"
  },
  "maintenance_signals": {
    "status": "ActivelyMaintained",
    "last_update_days": 3,
    "last_update_for_target_version": "2026-02-10T14:22:00Z",
    "update_frequency_days": 15,
    "notes": "Regular updates with latest Minecraft versions"
  },
  "adoption_risks": [],
  "discovery_evidence": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440001",
      "relationship_type": "Dependency",
      "related_mod_name": "Fabric API",
      "evidence_source": "ProviderApi",
      "confidence": 0.99,
      "discovered_at": "2026-02-13T10:45:32.123Z",
      "context": "Sodium requires Fabric API as a core dependency",
      "metadata": {
        "version_constraint": ">=0.90.0"
      }
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "relationship_type": "Incompatibility",
      "related_mod_name": "OptiFine",
      "evidence_source": "VersionCompatibility",
      "confidence": 0.85,
      "discovered_at": "2026-02-13T10:45:40.456Z",
      "context": "Known conflict: Sodium and OptiFine both modify rendering. Use Sodium with Iris Shaders instead.",
      "metadata": {
        "conflict_type": "rendering_engine",
        "community_reports": 42
      }
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440003",
      "relationship_type": "Complement",
      "related_mod_name": "Iris Shaders",
      "evidence_source": "CommunityData",
      "confidence": 0.92,
      "discovered_at": "2026-02-13T10:46:00.123Z",
      "context": "Popular shader mod designed specifically for Sodium compatibility",
      "metadata": {
        "popularity_score": 0.96
      }
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440004",
      "relationship_type": "Complement",
      "related_mod_name": "Lithium",
      "evidence_source": "CommunityData",
      "confidence": 0.92,
      "discovered_at": "2026-02-13T10:46:05.789Z",
      "context": "Community recommendation: Sodium + Lithium provides optimization synergy"
    }
  ],
  "aggregated_metrics": {
    "total_downloads": 46600000,
    "max_followers": 15000,
    "earliest_created": "2020-08-15T00:00:00Z",
    "most_recent_update": "2026-02-10T14:22:00Z",
    "all_supported_versions": ["1.20.1", "1.20", "1.19.2", "1.18.2", "1.17.1"],
    "all_supported_loaders": ["Fabric"]
  },
  "decision_summary": {
    "recommendation": "Strongly Recommended",
    "reasoning": [
      "Actively maintained with frequent updates",
      "Over 46M downloads across all platforms",
      "Perfect compatibility with target Minecraft version 1.20.1 and Fabric",
      "Open source (LGPL-3.0) with transparent development",
      "No critical adoption risks identified",
      "Excellent synergy with complementary mods (Lithium, Iris Shaders)"
    ],
    "considerations": [
      "Incompatible with OptiFine; use Iris Shaders instead",
      "Requires Fabric API dependency"
    ]
  },
  "_links": {
    "self": "/api/v1/mods/660e8400-e29b-41d4-a716-446655440001",
    "modrinth": "https://modrinth.com/mod/sodium",
    "curseforge": "https://www.curseforge.com/minecraft/mods/sodium",
    "github": "https://github.com/CaffeineMC/sodium-fabric"
  }
}
```

**Status Codes**:
- **200 OK**: Mod details retrieved
- **404 Not Found**: Mod ID not found
- **410 Gone**: Mod archived (>30 days old session)

---

### 5. POST /api/v1/sessions

**Summary**: Save a search session with shortlisted mods and notes for team review.

**References**: FR-011, FR-012

**Request**:
```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "query_description": "1.20.1 Fabric performance mods for server optimization",
  "shortlisted_mod_ids": [
    "660e8400-e29b-41d4-a716-446655440001",
    "660e8400-e29b-41d4-a716-446655440002",
    "660e8400-e29b-41d4-a716-446655440003"
  ],
  "comparison_notes": {
    "660e8400-e29b-41d4-a716-446655440001": "Primary choice for rendering optimization",
    "660e8400-e29b-41d4-a716-446655440002": "Essential companion for general optimization",
    "660e8400-e29b-41d4-a716-446655440003": "Alternative if Iris Shaders needed"
  }
}
```

**Request Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `query_id` | `uuid` | Yes | Reference to completed search |
| `query_description` | `string` | No | User-provided label for session (max 200 chars) |
| `shortlisted_mod_ids` | `uuid[]` | Yes | ConsolidatedModProfile IDs to preserve (1-50 mods) |
| `comparison_notes` | `map` | No | Per-mod notes (max 500 chars each) |

**Response** (201 Created):
```json
{
  "session_id": "440e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-02-13T11:30:00.000Z",
  "updated_at": "2026-02-13T11:30:00.000Z",
  "expires_at": "2026-03-15T11:30:00.000Z",
  "status": "active",
  "original_query": {
    "keywords": ["fabric", "performance"],
    "filters": {
      "minecraft_version": "1.20.1",
      "loaders": ["Fabric"]
    }
  },
  "shortlisted_mods": [
    "660e8400-e29b-41d4-a716-446655440001",
    "660e8400-e29b-41d4-a716-446655440002",
    "660e8400-e29b-41d4-a716-446655440003"
  ],
  "total_candidates_found": 83,
  "search_duration_ms": 12453,
  "providers_used": ["Modrinth", "CurseForge"],
  "shareable_link": "https://mcm-finder.example.com/session/440e8400",
  "_links": {
    "self": "/api/v1/sessions/440e8400-e29b-41d4-a716-446655440000",
    "retrieve": "/api/v1/sessions/440e8400-e29b-41d4-a716-446655440000",
    "share": "https://mcm-finder.example.com/session/440e8400"
  }
}
```

**Status Codes**:
- **201 Created**: Session saved successfully
- **400 Bad Request**: Invalid query_id or shortlisted_mod_ids
- **404 Not Found**: Query ID not found
- **429 Too Many Requests**: Exceeded session creation limit

---

### 6. GET /api/v1/sessions/{id}

**Summary**: Retrieve a saved search session for team review and comparison.

**References**: FR-011, FR-012

**Request Parameters**:
```
GET /api/v1/sessions/440e8400-e29b-41d4-a716-446655440000
```

**Response** (200 OK):
```json
{
  "session_id": "440e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-02-13T11:30:00.000Z",
  "updated_at": "2026-02-13T11:30:00.000Z",
  "expires_at": "2026-03-15T11:30:00.000Z",
  "status": "active",
  "query_description": "1.20.1 Fabric performance mods for server optimization",
  "original_query": {
    "keywords": ["fabric", "performance"],
    "filters": {
      "minecraft_version": "1.20.1",
      "loaders": ["Fabric"],
      "min_downloads": 1000
    },
    "provider_scope": ["Modrinth", "CurseForge"],
    "sort_mode": "UpdateRecencyVersionAware"
  },
  "shortlisted_mods": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "canonical_name": "Sodium",
      "composite_relevance": 0.98,
      "note": "Primary choice for rendering optimization",
      "compatibility_assessment": "FullyCompatible",
      "maintenance_signals": {
        "status": "ActivelyMaintained",
        "last_update_days": 3
      },
      "adoption_risks": []
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "canonical_name": "Lithium",
      "composite_relevance": 0.94,
      "note": "Essential companion for general optimization",
      "compatibility_assessment": "FullyCompatible",
      "maintenance_signals": {
        "status": "ActivelyMaintained",
        "last_update_days": 7
      },
      "adoption_risks": []
    }
  ],
  "search_metadata": {
    "total_duration_ms": 12453,
    "total_candidates_found": 83,
    "providers_used": ["Modrinth", "CurseForge"],
    "discovery_completed": true
  },
  "shareable_link": "https://mcm-finder.example.com/session/440e8400",
  "_links": {
    "self": "/api/v1/sessions/440e8400-e29b-41d4-a716-446655440000",
    "mod_details": "/api/v1/mods/{mod_id}",
    "delete": "/api/v1/sessions/440e8400-e29b-41d4-a716-446655440000",
    "share": "https://mcm-finder.example.com/session/440e8400"
  }
}
```

**Status Codes**:
- **200 OK**: Session retrieved
- **404 Not Found**: Session ID not found or expired
- **410 Gone**: Session archived (>30 days)

---

### 7. GET /api/v1/health

**Summary**: Check system health and provider availability.

**References**: NFR-003

**Request Parameters**:
```
GET /api/v1/health
```

**Response** (200 OK):
```json
{
  "status": "healthy",
  "timestamp": "2026-02-13T11:35:00.000Z",
  "uptime_seconds": 3600,
  "version": "1.0.0",
  "providers": {
    "Modrinth": {
      "status": "operational",
      "response_time_ms": 145,
      "last_check": "2026-02-13T11:35:00.000Z",
      "availability_percent": 99.8
    },
    "CurseForge": {
      "status": "operational",
      "response_time_ms": 312,
      "last_check": "2026-02-13T11:35:00.000Z",
      "availability_percent": 98.2
    }
  },
  "database": {
    "status": "healthy",
    "response_time_ms": 2
  },
  "cache": {
    "status": "healthy",
    "hit_rate_percent": 65.3,
    "entries": 1250
  },
  "_links": {
    "status_page": "https://status.example.com"
  }
}
```

**Response Variant** (Degraded - 200 OK):
```json
{
  "status": "degraded",
  "timestamp": "2026-02-13T11:35:00.000Z",
  "message": "CurseForge temporarily unavailable. Searches limited to Modrinth only.",
  "providers": {
    "Modrinth": {
      "status": "operational"
    },
    "CurseForge": {
      "status": "degraded",
      "last_error": "connection_timeout",
      "error_time": "2026-02-13T11:34:15.000Z",
      "time_to_retry_seconds": 60
    }
  }
}
```

**Status Codes**:
- **200 OK**: System healthy (status: healthy | degraded)
- **503 Service Unavailable**: System unavailable (all providers down)

---

## Data Models

### Risk Factors

Risk factors returned in consolidated mod details:

```json
{
  "risk_type": "OutdatedVersion",
  "description": "No updates for target Minecraft version in 8+ months",
  "severity": "Medium"
}
```

**RiskType Values**:
- `LicenseConflict`: Incompatible or conflicting license
- `DependencyIssue`: Missing or incompatible dependencies
- `ProviderDiscrepancy`: Major differences between provider metadata
- `OutdatedVersion`: No recent updates for target MC version
- `LowAdoption`: Very few downloads or followers

**RiskSeverity Values**:
- `Low`: Informational; does not prevent use
- `Medium`: Should review before using
- `High`: Strongly recommend avoiding or investigating further

---

## Error Handling

### Standard Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "field_name",
      "constraint": "constraint_type",
      "value": "provided_value"
    },
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-02-13T11:30:00.000Z"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_QUERY` | 400 | Query validation failed (missing keywords, invalid version, etc.) |
| `PROVIDER_ERROR` | 503 | One or more providers unavailable |
| `TIMEOUT` | 504 | Search exceeded discovery timeout |
| `RATE_LIMITED` | 429 | Client exceeded rate limit |
| `NOT_FOUND` | 404 | Query/session/mod ID not found |
| `INTERNAL_ERROR` | 500 | Unexpected server error |
| `VALIDATION_ERROR` | 400 | Request body validation failed |

---

## Rate Limiting

**Policy**: 30 requests per minute per IP address or authenticated user

**Headers Returned**:
```
RateLimit-Limit: 30
RateLimit-Remaining: 25
RateLimit-Reset: 1707829260
```

**When Limit Exceeded** (429 Too Many Requests):
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded: 30 requests per minute",
    "retry_after_seconds": 45
  }
}
```

---

## Versioning

**API Version**: 1.0 (in path: `/api/v1/`)

**Versioning Strategy**:
- Major versions in URL path (`/v1/`, `/v2/`, etc.)
- Backwards-incompatible changes trigger major version bump
- New endpoints added without version bump
- Deprecated endpoints marked with `X-Deprecation-Warning` header

**Future Version** (Planned):
- `/api/v2/` with additional provider support (GitHub, mc百科)
- Enhanced discovery algorithms
- Advanced filtering for mod conflicts

---

## Authentication & Caching

**Authentication**: Optional
- Anonymous users: Per-IP rate limiting and caching
- Authenticated users: Per-user rate limiting and personalized caching
- Upstream provider note: CurseForge adapter requires `x-api-key`; backend MUST surface degraded-mode
  notice if key is missing or rejected.

**Caching**:
- Provider responses: 24-hour TTL
- Search results: 6-hour TTL (query_id-specific)
- Session data: 30-day TTL
- Client-local sessions: 7-day TTL with freshness metadata (`cached_at`, `expires_at`, `cache_version`)

**Cache Headers**:
- Successful searches: `Cache-Control: public, max-age=21600`
- Health checks: `Cache-Control: public, max-age=60`
- Session data: `Cache-Control: private, max-age=2592000`
- Result/detail endpoints: `ETag` and `Last-Modified` headers for conditional client fetches
- Cache diagnostics: `X-Cache-Status: hit|miss|stale-revalidated`

---

## Pagination

All list endpoints support cursor-based pagination:

**Query Parameters**:
```
?offset=0&limit=20
```

**Response Metadata**:
```json
{
  "pagination": {
    "offset": 0,
    "limit": 20,
    "total": 1250,
    "has_more": true
  },
  "_links": {
    "self": "/api/v1/search/xyz/results?offset=0&limit=20",
    "next": "/api/v1/search/xyz/results?offset=20&limit=20",
    "prev": null
  }
}
```

---

## Request/Response Examples

### Example 1: Complete Search Workflow

**Step 1: Initiate Search**
```bash
curl -X POST http://localhost:3000/api/v1/search \
  -H "Content-Type: application/json" \
  -d '{
    "keywords": ["rendering", "shader"],
    "filters": {
      "minecraft_version": "1.20.1",
      "loaders": ["Fabric"]
    },
    "provider_scope": ["Modrinth", "CurseForge"],
    "enable_deep_discovery": true
  }'
```

Response (202 Accepted):
```json
{
  "query_id": "abc12345...",
  "status": "queued",
  "_links": {
    "progress": "/api/v1/search/abc12345.../discovery"
  }
}
```

**Step 2: Poll Progress**
```bash
curl http://localhost:3000/api/v1/search/abc12345.../discovery
```

Response (200 OK):
```json
{
  "status": "discovering",
  "progress": {
    "percentage": 75,
    "estimated_remaining_ms": 5000
  }
}
```

**Step 3: Retrieve Results**
```bash
curl http://localhost:3000/api/v1/search/abc12345.../results?limit=10
```

Response (200 OK):
```json
{
  "consolidated_mods": [
    { "canonical_name": "Iris Shaders", "composite_relevance": 0.96 },
    { "canonical_name": "Complementary Shaders", "composite_relevance": 0.92 }
  ]
}
```

---

## Implementation Notes

1. **Search Pipeline** (FR-001, FR-002):
   - Submit to `/search` endpoint (returns immediately with query_id)
   - Poll `/search/{query_id}/discovery` for progress
   - Retrieve consolidated results via `/search/{query_id}/results`

2. **Deep Discovery** (FR-006, FR-007):
   - Results **never** published until discovery completes
   - Progress tracked via `/discovery` endpoint
   - Timeout configurable per request

3. **Degraded Coverage** (NFR-003):
   - Provider failures detected within 10 seconds
   - Degraded notices returned in progress and results
   - Partial results include notice of missing coverage

4. **Conflict Resolution** (FR-008):
   - Metadata conflicts explicitly returned
   - Modrinth preferred as primary source for tie-breaking
   - Users shown both provider values

5. **Session Preservation** (FR-011, FR-012):
   - Sessions auto-expire after 30 days
   - Shareable links generated for team review
   - Per-mod notes preserved with shortlist

---

## Testing Scenarios

### Scenario 1: Normal Search
- Both providers available
- 50-100 results after filtering
- Discovery completes in ~10 seconds
- All metadata consistent

### Scenario 2: Provider Failure
- CurseForge timeout after 10 seconds
- System continues with Modrinth-only results
- Degraded notice shown immediately
- Coverage: 1/2 providers

### Scenario 3: Conflicting Metadata
- Same mod has different descriptions/versions between providers
- Conflicts returned with resolution strategy
- User shown both values for critical fields

### Scenario 4: Large Result Set
- Query returns 500+ results
- Pagination applied (20 per page)
- Search still completes within 30 seconds
- Deep discovery continues in background

---

## Performance Targets (NFR-001)

- POST /search: < 200ms response time
- GET /discovery: < 100ms response time
- GET /results: < 500ms response time (paginated)
- GET /mods/{id}: < 100ms response time
- Complete search with discovery: < 30 seconds (95% of queries)

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-13  
**Status**: Implementation-Ready
