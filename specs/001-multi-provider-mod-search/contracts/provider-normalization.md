# Provider Normalization Contract: Multi-Provider Mod Finder

**Feature**: 001-multi-provider-mod-search  
**Date**: 2026-02-13  
**Status**: Implementation-Ready  

---

## Overview

This contract defines the schema mappings and validation rules for normalizing Modrinth and CurseForge API responses into the `ProviderResultRecord` and `ConsolidatedModProfile` data structures. It also specifies conflict detection rules and test cases for schema validation.

**Scope**:
- Modrinth API v3 → NormalizedModResult mapping
- CurseForge API v2 → NormalizedModResult mapping
- Required/optional fields per provider
- Conflict detection and resolution rules
- Validation test cases

---

## Data Normalization Model

### NormalizedModResult (Internal)

This intermediate structure represents a single mod result from any provider, normalized to a common format:

```rust
struct NormalizedModResult {
    // Identity
    canonical_mod_id: String,          // Provider-specific ID
    source: Provider,                  // Modrinth | CurseForge
    name: String,                      // Mod name
    slug: String,                      // URL-safe identifier
    
    // Core metadata
    author: String,                    // Primary author/team
    summary: String,                   // Short description (<500 chars)
    description: Option<String>,       // Full description (HTML)
    
    // Compatibility
    supported_versions: Vec<String>,   // Minecraft versions
    supported_loaders: Vec<ModLoader>, // Fabric, Forge, Quilt, NeoForge
    
    // Metrics
    downloads: u64,                    // Total downloads
    followers: Option<u64>,            // Followers/subscribers
    last_updated: DateTime<Utc>,       // Latest version release
    created_at: DateTime<Utc>,         // Project creation date
    
    // Metadata
    categories: Vec<String>,           // Tags/categories
    license: Option<String>,           // License (SPDX identifier)
    source_url: Option<String>,        // Source code repository
    
    // Provider-specific
    provider_url: String,              // Direct link to mod page
    provider_metadata: serde_json::Value, // Raw provider data (for edge cases)
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
}
```

---

## Provider Mappings

### 1. Modrinth API → NormalizedModResult

**API Documentation**: https://docs.modrinth.com/api-spec/

#### Search Endpoint Mapping

**Modrinth API Response (Search)**:
```json
{
  "hits": [
    {
      "project_id": "AANobbMI",
      "project_type": "mod",
      "slug": "sodium",
      "title": "Sodium",
      "description": "Modern rendering engine and client-side optimization mod for Minecraft",
      "categories": ["optimization", "rendering"],
      "display_categories": ["optimization", "rendering"],
      "author": "CaffeineMC",
      "icon_url": "https://cdn.modrinth.com/data/AANobbMI/icon.png",
      "downloads": 24500000,
      "follows": 15000,
      "date_created": "2020-08-15T00:00:00Z",
      "date_modified": "2026-02-10T14:22:00Z",
      "latest_version": "0.5.5",
      "license": "LGPL-3.0",
      "client_side": "required",
      "server_side": "optional",
      "versions": ["1.20.1", "1.20", "1.19.2", "1.18.2", "1.17.1"],
      "loaders": ["fabric"],
      "gallery": []
    }
  ]
}
```

**Normalization Mapping**:

| Modrinth Field | NormalizedModResult Field | Notes |
|---|---|---|
| `project_id` | `canonical_mod_id` | Required |
| `slug` | `slug` | Required; URL-safe |
| `title` | `name` | Required; max 100 chars |
| `description` | `summary` | Required; truncate to 500 chars if longer |
| N/A | `description` | Full description fetched via `/projects/{id}` endpoint (optional) |
| `author` | `author` | Required; primary author |
| `categories` | `categories` | Array of tags |
| `downloads` | `downloads` | Required; non-negative |
| `follows` | `followers` | Optional; represents followers |
| `date_created` | `created_at` | Required; UTC datetime |
| `date_modified` | `last_updated` | Required; UTC datetime |
| `license` | `license` | Optional; SPDX identifier |
| `versions` | `supported_versions` | Required; array of version strings |
| `loaders` | `supported_loaders` | Required; normalize loader names (see Loader Mapping) |
| Provider URL | `provider_url` | `https://modrinth.com/mod/{slug}` |

**Project Details Endpoint** (for full descriptions):
```
GET https://api.modrinth.com/v2/project/{id}
```

Returns additional fields:
- `body` (HTML): Full description (map to `description`)
- `source_url`: Source repository link
- `team`: Array of team members (use primary author from `author` field)
- `status`: "approved" | "rejected" | "draft" | "archived"

**Loader Mapping**:
- `"fabric"` → `ModLoader::Fabric`
- `"forge"` → `ModLoader::Forge`
- `"quilt"` → `ModLoader::Quilt`
- `"neoforge"` → `ModLoader::NeoForge`

**Validation Rules**:
- `project_id` must be non-empty, alphanumeric
- `title` (name) must be non-empty, ≤100 characters
- `description` must be non-empty, ≤500 characters
- `downloads` must be non-negative integer
- `date_created` ≤ `date_modified`
- `versions` must contain valid semantic versions or Minecraft version identifiers
- `loaders` array must not be empty

---

### 2. CurseForge API → NormalizedModResult

**API Documentation**: https://docs.curseforge.com/

#### Search Endpoint Mapping

**CurseForge API Response (Search)**:
```json
{
  "data": [
    {
      "id": 394468,
      "name": "Sodium",
      "summary": "Modern rendering engine for Minecraft (client-side)",
      "description": "Sodium is a modern rendering engine for Minecraft...",
      "websiteUrl": "https://www.curseforge.com/minecraft/mods/sodium",
      "logoUrl": {
        "url": "https://media.forgecdn.net/avatars/394/468/637378123456789012.png"
      },
      "primaryAuthorName": "CaffeineMC",
      "categories": [
        {
          "id": 410,
          "name": "Optimization",
          "url": "https://www.curseforge.com/minecraft/mods?filter-cat-ids=410",
          "avatarUrl": "https://media.forgecdn.net/avatars/410/x/637378123456789012.png"
        }
      ],
      "authors": [
        {
          "id": 123456,
          "name": "CaffeineMC",
          "url": "https://www.curseforge.com/members/caffeinmc"
        }
      ],
      "downloadCount": 22100000,
      "dateCreated": "2020-08-17T00:00:00Z",
      "dateModified": "2026-02-09T10:00:00Z",
      "dateReleased": "2026-02-09T10:00:00Z",
      "slug": "sodium",
      "gamePopularityRank": 15,
      "isAvailable": true,
      "gameId": 432,
      "classId": 6,
      "modLoaders": [
        {
          "name": "Fabric",
          "gameVersion": "1.20.1"
        }
      ],
      "isServerPack": false,
      "isFeatured": true,
      "trendingScore": 98.5,
      "license": {
        "name": "LGPL-3.0",
        "url": "https://www.gnu.org/licenses/lgpl-3.0.en.html",
        "mirrorsUrl": null,
        "sourceUrl": null
      },
      "links": {
        "websiteUrl": "https://www.curseforge.com/minecraft/mods/sodium",
        "wikiUrl": null,
        "issuesUrl": null,
        "sourceUrl": "https://github.com/CaffeineMC/sodium-fabric"
      },
      "latestFilesIndexes": [
        {
          "gameVersion": "1.20.1",
          "fileMainClassification": "Release",
          "modLoader": 4,
          "index": 5
        }
      ]
    }
  ]
}
```

**Normalization Mapping**:

| CurseForge Field | NormalizedModResult Field | Notes |
|---|---|---|
| `id` | `canonical_mod_id` | Required; convert to string |
| `slug` | `slug` | Required; URL-safe |
| `name` | `name` | Required; max 100 chars |
| `summary` | `summary` | Required; max 500 chars (use instead of `description` for shorter form) |
| `description` | `description` | Optional; full description |
| `primaryAuthorName` | `author` | Required; primary author name |
| `categories[].name` | `categories` | Array of category names |
| `downloadCount` | `downloads` | Required; non-negative |
| N/A | `followers` | Not provided by CurseForge API (omit) |
| `dateCreated` | `created_at` | Required; UTC datetime |
| `dateModified` | `last_updated` | Required; UTC datetime (use `dateReleased` if more recent) |
| `license.name` | `license` | Optional; SPDX identifier |
| `links.sourceUrl` | `source_url` | Optional; source repository |
| `websiteUrl` | `provider_url` | Required; direct link to mod page |

**Game Version Mapping** (via `/mods/{modId}/files`):
```
GET https://api.curseforge.com/v1/mods/{modId}/files
```

Extract supported versions from file metadata:
- `files[].gameVersions` array contains supported Minecraft versions
- Aggregate all unique versions across all files
- Filter to only "Release" and "Beta" classifications

**Loader Mapping**:
- `"Fabric"` → `ModLoader::Fabric`
- `"Forge"` → `ModLoader::Forge`
- `"Quilt"` → `ModLoader::Quilt`
- `"NeoForge"` → `ModLoader::NeoForge`

**Validation Rules**:
- `id` must be positive integer
- `name` must be non-empty, ≤100 characters
- `summary` must be non-empty, ≤500 characters
- `downloadCount` must be non-negative integer
- `dateCreated` ≤ `dateModified`
- `slug` must be non-empty, URL-safe
- `isAvailable` must be true (skip archived/deleted mods)

---

## Loader Normalization Rules

Both providers use different loader naming conventions. Normalize to enum:

| Modrinth | CurseForge | Standard Name | Enum Value |
|---|---|---|---|
| `fabric` | `Fabric` | Fabric | `Fabric` |
| `forge` | `Forge` | Forge | `Forge` |
| `quilt` | `Quilt` | Quilt | `Quilt` |
| `neoforge` | `NeoForge` | NeoForge | `NeoForge` |

**Handling Edge Cases**:
- If loader string case doesn't match exactly, normalize to title case then compare
- Unknown loader values: Log warning and skip (do not add to loaders array)
- Empty loaders array: Reject record (requires at least one loader)

---

## Version Normalization Rules

Minecraft versions come in various formats:

| Format | Example | Normalization |
|---|---|---|
| Full semantic | `1.20.1` | Keep as-is |
| Snapshot | `23w03a` | Keep as-is |
| Pre-release | `1.20-rc.1` | Keep as-is |
| Approximate | `1.20.x` | Expand to all known 1.20.* versions |
| Range | `1.20+` | Treat as specific version only (no range expansion) |

**Validation**:
- All version strings must match pattern: `\d+\.\d+(\.\d+)?([a-z0-9\-\.]+)?`
- Invalid versions: Log warning and skip
- Empty versions array: Reject record

---

## Conflict Detection Rules

When the same mod appears in both Modrinth and CurseForge, the system must detect and flag conflicts:

### Matching Criteria (to identify same mod across providers)

```
Same Mod If:
  (slug matches OR
   (name similarity > 0.95 AND author matches)) AND
  (at least one supported version overlaps OR
   created_at dates within 30 days)
```

**Slug Matching**:
- Both providers publish slug; exact match is authoritative
- Modrinth slug is usually more stable

**Name Matching**:
- Normalize both names: lowercase, remove special chars, strip whitespace
- Use Levenshtein distance; threshold 0.95 similarity
- Example: "Sodium" vs "sodium-fabric" → match (similarity 0.93, likely same)

**Version Overlap**:
- At least one Minecraft version in common between both providers
- Prevents false positives for unrelated mods with similar names

**Creation Date Proximity**:
- Fallback if no version overlap
- Dates within 30 days suggest same project

### Conflict Types

| Conflict | Fields | Severity | Resolution |
|---|---|---|---|
| **Version Mismatch** | `supported_versions` | Minor | Use union of all versions |
| **Loader Mismatch** | `supported_loaders` | Minor | Use union of all loaders |
| **Download Count Disparity** | `downloads` | Minor | Sum values (caveat: possible double-counting) |
| **Author Mismatch** | `author` | Major | Investigate; flag for manual review if diff > 50% edit distance |
| **Description Mismatch** | `summary`, `description` | Minor | Keep both; show user |
| **License Conflict** | `license` | Critical | Both licenses must be compatible (see License Compatibility Matrix) |
| **Created Date Discrepancy** | `created_at` | Minor | Use earlier date |
| **Last Updated Discrepancy** | `last_updated` | Minor | Use more recent date |

### License Compatibility Matrix

```
Compatible Pairs:
  - MIT + MIT = ✓
  - MIT + Apache-2.0 = ✓
  - LGPL-3.0 + GPL-3.0 = ✓
  - Apache-2.0 + Apache-2.0 = ✓
  - GPL-3.0 + AGPL-3.0 = ✓

Incompatible Pairs:
  - GPL-2.0 + Apache-2.0 = ✗ (conflicting terms)
  - LGPL-2.1 + GPL-3.0 = ✗ (version mismatch)
  - Proprietary + MIT = ✗

Unknown/Custom Licenses:
  - Flag as "Requires Review" (severity: Medium)
```

---

## Schema Validation Test Cases

### Test Case 1: Valid Modrinth Search Result

**Input** (Modrinth API response):
```json
{
  "project_id": "AANobbMI",
  "slug": "sodium",
  "title": "Sodium",
  "description": "Modern rendering engine and client-side optimization mod for Minecraft",
  "author": "CaffeineMC",
  "categories": ["optimization", "rendering"],
  "downloads": 24500000,
  "follows": 15000,
  "date_created": "2020-08-15T00:00:00Z",
  "date_modified": "2026-02-10T14:22:00Z",
  "license": "LGPL-3.0",
  "versions": ["1.20.1", "1.20", "1.19.2"],
  "loaders": ["fabric"]
}
```

**Expected Output** (NormalizedModResult):
```
✓ canonical_mod_id: "AANobbMI"
✓ source: Modrinth
✓ name: "Sodium"
✓ slug: "sodium"
✓ author: "CaffeineMC"
✓ summary: "Modern rendering engine and client-side optimization mod for Minecraft"
✓ downloads: 24500000
✓ followers: Some(15000)
✓ created_at: 2020-08-15T00:00:00Z
✓ last_updated: 2026-02-10T14:22:00Z
✓ supported_versions: ["1.20.1", "1.20", "1.19.2"]
✓ supported_loaders: [Fabric]
✓ categories: ["optimization", "rendering"]
✓ license: Some("LGPL-3.0")
✓ provider_url: "https://modrinth.com/mod/sodium"
```

**Validation Status**: ✓ PASS

---

### Test Case 2: Valid CurseForge Search Result

**Input** (CurseForge API response):
```json
{
  "id": 394468,
  "slug": "sodium",
  "name": "Sodium",
  "summary": "Modern rendering engine for Minecraft (client-side)",
  "primaryAuthorName": "CaffeineMC",
  "categories": [{"name": "Optimization"}],
  "downloadCount": 22100000,
  "dateCreated": "2020-08-17T00:00:00Z",
  "dateModified": "2026-02-09T10:00:00Z",
  "license": {"name": "LGPL-3.0"},
  "websiteUrl": "https://www.curseforge.com/minecraft/mods/sodium",
  "links": {
    "sourceUrl": "https://github.com/CaffeineMC/sodium-fabric"
  }
}
```

**Expected Output** (NormalizedModResult):
```
✓ canonical_mod_id: "394468"
✓ source: CurseForge
✓ name: "Sodium"
✓ slug: "sodium"
✓ author: "CaffeineMC"
✓ summary: "Modern rendering engine for Minecraft (client-side)"
✓ downloads: 22100000
✓ followers: None
✓ created_at: 2020-08-17T00:00:00Z
✓ last_updated: 2026-02-09T10:00:00Z
✓ supported_versions: [retrieved separately from /files endpoint]
✓ supported_loaders: [retrieved separately from /files endpoint]
✓ categories: ["Optimization"]
✓ license: Some("LGPL-3.0")
✓ source_url: Some("https://github.com/CaffeineMC/sodium-fabric")
✓ provider_url: "https://www.curseforge.com/minecraft/mods/sodium"
```

**Validation Status**: ✓ PASS

---

### Test Case 3: Missing Required Field (Modrinth)

**Input** (invalid - missing `loaders`):
```json
{
  "project_id": "test123",
  "title": "Test Mod",
  "description": "Test description",
  "author": "TestAuthor",
  "downloads": 1000,
  "date_created": "2025-01-01T00:00:00Z",
  "date_modified": "2026-01-01T00:00:00Z",
  "versions": ["1.20.1"],
  "loaders": []
}
```

**Expected Behavior**:
- ✗ Validation fails
- Reason: `loaders` array is empty (requires at least one)
- Action: Skip record, log error

**Error Message**:
```
Schema validation failed: Modrinth mod 'test123'
Field: loaders
Reason: Empty array; at least one loader required
```

---

### Test Case 4: Invalid Version Format

**Input**:
```json
{
  "project_id": "AANobbMI",
  "title": "Sodium",
  "author": "CaffeineMC",
  "downloads": 24500000,
  "date_created": "2020-08-15T00:00:00Z",
  "date_modified": "2026-02-10T14:22:00Z",
  "versions": ["1.20.1", "1.20", "invalid-version", "2026.01.20"],
  "loaders": ["fabric"]
}
```

**Expected Behavior**:
- Parse valid versions: ["1.20.1", "1.20"]
- Skip invalid versions: ["invalid-version", "2026.01.20"]
- Log warnings for skipped versions
- Proceed with valid versions

**Result**:
```
✓ supported_versions: ["1.20.1", "1.20"]
⚠ Warnings:
  - Skipped version "invalid-version": does not match version pattern
  - Skipped version "2026.01.20": likely invalid date format
```

---

### Test Case 5: Slug Normalization

**Input** (Modrinth title with special characters):
```json
{
  "project_id": "xyz789",
  "title": "Advanced Rendering Engine - HD Textures & Shaders",
  "slug": "advanced-rendering-engine-hd-textures-shaders",
  "author": "DevTeam",
  "downloads": 5000,
  "date_created": "2023-01-01T00:00:00Z",
  "date_modified": "2025-12-01T00:00:00Z",
  "versions": ["1.20.1"],
  "loaders": ["fabric"]
}
```

**Expected Output**:
```
✓ slug: "advanced-rendering-engine-hd-textures-shaders"
✓ name: "Advanced Rendering Engine - HD Textures & Shaders" (truncate to 100 chars if needed)
```

**Slug Validation Rules**:
- Must contain only alphanumeric, hyphens, underscores
- Hyphens for word separation
- Must not start/end with hyphen
- ✓ Valid: `sodium`, `mod-name`, `mod_name_123`
- ✗ Invalid: `-mod`, `mod-`, `mod name`, `mod@name`

---

### Test Case 6: Conflict Detection - Same Mod

**Modrinth Result**:
```json
{
  "project_id": "AANobbMI",
  "slug": "sodium",
  "title": "Sodium",
  "author": "CaffeineMC",
  "downloads": 24500000,
  "date_created": "2020-08-15T00:00:00Z",
  "date_modified": "2026-02-10T14:22:00Z",
  "versions": ["1.20.1", "1.20", "1.19.2"],
  "loaders": ["fabric"]
}
```

**CurseForge Result**:
```json
{
  "id": 394468,
  "slug": "sodium",
  "name": "Sodium",
  "primaryAuthorName": "CaffeineMC",
  "downloadCount": 22100000,
  "dateCreated": "2020-08-17T00:00:00Z",
  "dateModified": "2026-02-09T10:00:00Z"
}
```

**Expected Conflict Detection**:
```
✓ Identified as same mod (slug match: "sodium")
✓ Conflicts detected:

  1. Version Mismatch
     Severity: Minor
     Values:
       Modrinth: ["1.20.1", "1.20", "1.19.2"]
       CurseForge: [retrieved separately]
     Resolution: Union of all versions

  2. Download Count Disparity
     Severity: Minor
     Values:
       Modrinth: 24,500,000
       CurseForge: 22,100,000
     Resolution: Sum with caveat (possible double-counting)

  3. Last Updated Mismatch
     Severity: Minor
     Values:
       Modrinth: 2026-02-10T14:22:00Z
       CurseForge: 2026-02-09T10:00:00Z
     Resolution: Use more recent (Modrinth)

✓ Consolidated Result:
  - name: "Sodium"
  - total_downloads: 46,600,000 (caveat: may include duplicates)
  - last_updated: 2026-02-10T14:22:00Z
  - supported_versions: [union]
  - confidence_score: 0.95 (high - same slug, author, versions overlap)
```

---

### Test Case 7: Conflict Detection - Incompatible Licenses

**Modrinth Result**:
```json
{
  "project_id": "mod1",
  "slug": "mod-a",
  "title": "Mod A",
  "license": "GPL-2.0",
  "author": "Author1",
  "downloads": 1000,
  "date_created": "2020-01-01T00:00:00Z",
  "date_modified": "2025-01-01T00:00:00Z",
  "versions": ["1.20.1"],
  "loaders": ["fabric"]
}
```

**CurseForge Result** (identified as same mod):
```json
{
  "id": 2,
  "slug": "mod-a",
  "name": "Mod A",
  "license": {"name": "Apache-2.0"},
  "primaryAuthorName": "Author1",
  "downloadCount": 900,
  "dateCreated": "2020-01-02T00:00:00Z",
  "dateModified": "2025-01-02T00:00:00Z"
}
```

**Expected Behavior**:
```
✓ Conflict Detected: License Incompatibility
  Severity: CRITICAL
  Values:
    Modrinth: GPL-2.0
    CurseForge: Apache-2.0
  Compatibility: Incompatible (conflicting terms)
  
Action:
  - Flag mod profile with critical risk
  - Show both licenses to user
  - Recommend contacting authors for clarification
  - confidence_score: 0.60 (lowered due to critical conflict)
  
Risk Factor:
  {
    "risk_type": "LicenseConflict",
    "description": "Conflicting license information: GPL-2.0 (Modrinth) vs Apache-2.0 (CurseForge)",
    "severity": "High"
  }
```

---

### Test Case 8: Conflict Detection - Different Mods

**Modrinth Result**:
```json
{
  "project_id": "mod1",
  "slug": "sodium",
  "title": "Sodium",
  "author": "CaffeineMC",
  "downloads": 24500000,
  "date_created": "2020-08-15T00:00:00Z",
  "date_modified": "2026-02-10T14:22:00Z",
  "versions": ["1.20.1", "1.20"],
  "loaders": ["fabric"]
}
```

**CurseForge Result**:
```json
{
  "id": 999,
  "slug": "sodium-custom",
  "name": "Sodium Custom Edition",
  "primaryAuthorName": "CustomTeam",
  "downloadCount": 50000,
  "dateCreated": "2023-06-01T00:00:00Z",
  "dateModified": "2024-12-01T00:00:00Z"
}
```

**Expected Behavior**:
```
✗ Not matched as same mod:
  - Slug mismatch: "sodium" ≠ "sodium-custom"
  - Author mismatch: "CaffeineMC" ≠ "CustomTeam"
  - Name similarity: 0.62 < 0.95 threshold
  - Creation date diff: 2.84 years > 30 day threshold
  
Result:
  - Treated as separate mods
  - No conflict detection
  - Both included in consolidated results
```

---

### Test Case 9: Missing Optional Fields

**Input** (Modrinth - minimal):
```json
{
  "project_id": "minimal123",
  "slug": "minimal-mod",
  "title": "Minimal Mod",
  "description": "A minimal mod",
  "author": "MinimalAuthor",
  "downloads": 100,
  "date_created": "2025-01-01T00:00:00Z",
  "date_modified": "2025-12-01T00:00:00Z",
  "versions": ["1.20.1"],
  "loaders": ["fabric"]
}
```

**Expected Output**:
```
✓ canonical_mod_id: "minimal123"
✓ name: "Minimal Mod"
✓ summary: "A minimal mod"
✓ author: "MinimalAuthor"
✓ downloads: 100
✓ followers: None (not provided, optional)
✓ supported_versions: ["1.20.1"]
✓ supported_loaders: [Fabric]
✓ categories: [] (not provided, optional)
✓ license: None (not provided, optional)
✓ source_url: None (not provided, optional)
✓ description: None (not provided, optional)

Status: ✓ PASS (all required fields present)
```

---

### Test Case 10: Provider Rate Limiting & Caching

**Scenario**: Successive searches for same mod

**Search 1** (fresh):
```
POST /search
  keywords: ["sodium"]
  providers: [Modrinth, CurseForge]
  
→ Calls:
  - Modrinth API search
  - CurseForge API search
→ Cache duration: 24 hours
```

**Search 2** (5 minutes later, same keyword):
```
POST /search
  keywords: ["sodium"]
  providers: [Modrinth, CurseForge]
  
→ Calls: (cached)
  - Modrinth API (from cache)
  - CurseForge API (from cache)
→ Response time: <100ms
→ No rate limit impact
```

**Expected Behavior**:
```
✓ Cache hit for both providers
✓ Response served from cache with < 100ms latency
✓ Cache header: Cache-Control: public, max-age=86400
✓ No additional API quota consumed
```

---

## Adapter Implementation Checklist

### Modrinth Adapter

- [ ] Implement search query builder
- [ ] Map search response to NormalizedModResult
- [ ] Fetch full descriptions via `/projects/{id}` endpoint
- [ ] Fetch game versions from `/projects/{id}/versions`
- [ ] Validate all required fields
- [ ] Handle pagination (search returns paginated results)
- [ ] Cache responses with 24h TTL
- [ ] Implement retry logic (exponential backoff)
- [ ] Log all validation errors
- [ ] Test with Modrinth live API

### CurseForge Adapter

- [ ] Implement search query builder
- [ ] Map search response to NormalizedModResult
- [ ] Fetch game versions from `/mods/{modId}/files`
- [ ] Fetch loader information from `/mods/{modId}/files`
- [ ] Validate all required fields
- [ ] Handle pagination (search returns paginated results)
- [ ] Cache responses with 24h TTL
- [ ] Implement retry logic (exponential backoff)
- [ ] Request API key management
- [ ] Log all validation errors
- [ ] Test with CurseForge live API

### Consolidation Engine

- [ ] Implement mod matching algorithm (slug-based, name-similarity-based)
- [ ] Detect all conflict types
- [ ] Apply license compatibility matrix
- [ ] Merge provider records
- [ ] Generate confidence scores
- [ ] Flag critical conflicts
- [ ] Create ProviderResultRecord instances
- [ ] Create ConsolidatedModProfile instances
- [ ] Test with test cases 1-10

---

## Error Handling

### Validation Errors

When a provider response fails schema validation:

1. **Log the error** with full context:
   ```
   ERROR: Schema validation failed
   Provider: Modrinth
   Mod ID: AANobbMI
   Field: loaders
   Reason: Empty array
   Raw Response: {...}
   ```

2. **Skip the record** (do not include in results)

3. **Track validation failures** for monitoring:
   ```
   validation_failure_rate = failed_records / total_records
   Alert if > 5%
   ```

4. **Continue processing** other records

### Network Errors

- **Timeout** (>10 seconds): Retry with exponential backoff
- **5xx errors**: Retry up to 3 times
- **4xx errors**: Skip provider (non-recoverable)
- **Connection refused**: Mark provider as unavailable

### Conflict Errors

- **License conflicts**: Flag with risk factor, continue processing
- **Version mismatches**: Use union, continue processing
- **Author conflicts**: Flag with medium risk, request manual review

---

## Testing Framework

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modrinth_normalization_valid() {
        // Test Case 1
    }

    #[test]
    fn test_curseforge_normalization_valid() {
        // Test Case 2
    }

    #[test]
    fn test_schema_validation_missing_loaders() {
        // Test Case 3
    }

    #[test]
    fn test_version_format_validation() {
        // Test Case 4
    }

    #[test]
    fn test_conflict_detection_same_mod() {
        // Test Case 6
    }

    #[test]
    fn test_conflict_detection_license_incompatible() {
        // Test Case 7
    }

    #[test]
    fn test_conflict_detection_different_mods() {
        // Test Case 8
    }

    #[test]
    fn test_optional_fields_handling() {
        // Test Case 9
    }
}
```

### Integration Tests

- [ ] Real API calls to Modrinth (staging environment)
- [ ] Real API calls to CurseForge (staging environment)
- [ ] Full consolidation pipeline
- [ ] Conflict detection with real data

### Performance Tests

- [ ] Normalization speed: <1ms per record
- [ ] Consolidation speed: <100ms for 100 mods
- [ ] Cache hit rate: >80%

---

## References

- **spec.md**: FR-001, FR-002, FR-003, FR-008
- **data-model.md**: ProviderResultRecord, ConsolidatedModProfile, MetadataConflict
- **rest-api.md**: POST /search, GET /mods/{id}

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-13  
**Status**: Implementation-Ready
