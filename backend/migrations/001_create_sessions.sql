-- Sessions database
-- Stores user search sessions and results for preservation

CREATE TABLE IF NOT EXISTS searches (
    id TEXT PRIMARY KEY,
    user_identifier TEXT,
    query_hash TEXT NOT NULL,
    query_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    expires_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_query_hash ON searches(query_hash);
CREATE INDEX IF NOT EXISTS idx_user_searches ON searches(user_identifier, created_at);

CREATE TABLE IF NOT EXISTS session_mods (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    mod_id TEXT NOT NULL,
    details_json TEXT NOT NULL,
    comparison_notes TEXT,
    FOREIGN KEY(session_id) REFERENCES searches(id)
);

CREATE INDEX IF NOT EXISTS idx_session_mods ON session_mods(session_id);

-- Enable WAL mode for concurrent access
PRAGMA journal_mode=WAL;
