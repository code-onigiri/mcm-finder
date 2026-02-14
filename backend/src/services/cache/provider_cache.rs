// Provider response cache (stub)
use sqlx::SqlitePool;

pub struct ProviderCache {
    pool: SqlitePool,
}

impl ProviderCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    pub async fn get(&self, _key: &str) -> Option<String> {
        // Stub - will be implemented in Phase 3
        None
    }
    
    pub async fn set(&self, _key: &str, _value: &str, _ttl_seconds: u64) -> Result<(), sqlx::Error> {
        // Stub - will be implemented in Phase 3
        Ok(())
    }
}
