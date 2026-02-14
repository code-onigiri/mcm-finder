-- Discovery cache database
-- Stores computed relationships and patterns for deep discovery

CREATE TABLE IF NOT EXISTS relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id_a TEXT NOT NULL,
    mod_id_b TEXT NOT NULL,
    rel_type TEXT NOT NULL,
    confidence REAL NOT NULL,
    evidence_json TEXT NOT NULL,
    computed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_relationships_mod ON relationships(mod_id_a, rel_type);
CREATE INDEX IF NOT EXISTS idx_relationships_confidence ON relationships(confidence DESC);

CREATE TABLE IF NOT EXISTS patterns (
    pattern_hash TEXT PRIMARY KEY,
    pattern_data TEXT NOT NULL,
    last_seen INTEGER NOT NULL
);

-- Enable WAL mode for concurrent access
PRAGMA journal_mode=WAL;
