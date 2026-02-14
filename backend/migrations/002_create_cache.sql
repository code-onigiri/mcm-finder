-- Provider cache database
-- Stores raw provider API responses for performance

CREATE TABLE IF NOT EXISTS provider_responses (
    cache_key TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    response_json TEXT NOT NULL,
    etag TEXT,
    cached_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_expiry ON provider_responses(provider, expires_at);
CREATE INDEX IF NOT EXISTS idx_provider_cache ON provider_responses(provider, cache_key);

-- Enable WAL mode for concurrent access
PRAGMA journal_mode=WAL;
