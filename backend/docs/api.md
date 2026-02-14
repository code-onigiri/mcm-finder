# MCM-Finder API Documentation

This document describes the current Axum HTTP routes exposed by `mcm-server`.

## Base URL

- Local: `http://localhost:3000`
- Prefix: `/api/v1`

## OpenAPI Skeleton (from current routes)

```yaml
openapi: 3.1.0
info:
  title: MCM-Finder API
  version: 0.1.0
paths:
  /api/v1/health:
    get: {}
  /api/v1/search:
    post: {}
  /api/v1/search/{id}/progress:
    get: {}
  /api/v1/mods/{id}:
    get: {}
  /api/v1/sessions:
    get: {}
    post: {}
  /api/v1/sessions/{id}:
    get: {}
```

## Endpoints

### `GET /api/v1/health`

Returns API status and provider readiness.

### `POST /api/v1/search`

Runs multi-provider search with filtering, sorting, and pagination.

Request fields:
- `keywords: string[]` (1-10 non-empty keywords)
- `filters.minecraft_version?: string` (`1.20`, `1.20.1`, `1.20.x`, `1.20+`)
- `filters.loaders?: string[]` (max 8)
- `filters.categories?: string[]` (max 25)
- `provider_scope?: string[]` (`modrinth`, `curseforge`)
- `sort_mode?: string` (`relevance`, `update-recency-version-aware`, `downloads`, `created`)
- `offset?: number`, `limit?: number` (limit clamped to 1..100)
- `force_refresh?: boolean`
- `search_id?: string` (UUID)

### `GET /api/v1/search/{id}/progress`

Returns progress stream via SSE (`event: progress` payloads).

### `GET /api/v1/mods/{id}`

Returns a consolidated profile and integrated summary for one cached mod result.

### `POST /api/v1/sessions`

Persists session shortlist and optional notes.

### `GET /api/v1/sessions/{id}`

Fetches one saved session by UUID.

### `GET /api/v1/sessions`

Lists recent sessions (`limit` query param, optional `user_identifier`).

## Error Format

All API errors use:

```json
{
  "error": {
    "code": "INVALID_QUERY",
    "message": "Human-readable details"
  }
}
```

Common codes:
- `INVALID_QUERY`
- `NOT_FOUND`
- `INTERNAL_ERROR`
- `RATE_LIMITED`

## Security Notes

- Per-client API throttling is enabled with `API_RATE_LIMIT_REQUESTS` + `API_RATE_LIMIT_WINDOW_SECONDS`.
- CurseForge key is loaded from environment (`CURSEFORGE_API_KEY`) and never written to logs.
