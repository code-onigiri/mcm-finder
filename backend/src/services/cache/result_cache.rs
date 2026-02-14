// T044: Consolidated result cache with sha256 cache keys and 24h TTL
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

pub struct ResultCache {
    pool: SqlitePool,
    ttl: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheQuery {
    pub providers: Vec<String>,
    pub query: String,
    pub filters: serde_json::Value,
}

impl ResultCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            ttl: Duration::hours(24),
        }
    }

    pub fn with_ttl(pool: SqlitePool, ttl: Duration) -> Self {
        Self { pool, ttl }
    }

    /// Generate cache key from query parameters using SHA256.
    pub fn generate_cache_key(cache_query: &CacheQuery) -> String {
        let mut hasher = Sha256::new();

        let mut providers = cache_query.providers.clone();
        providers.sort();

        hasher.update(providers.join(","));
        hasher.update(&cache_query.query);
        hasher.update(cache_query.filters.to_string());

        format!("{:x}", hasher.finalize())
    }

    /// Get cached consolidated results if they exist and haven't expired.
    pub async fn get(&self, key: &str) -> Option<String> {
        let row = sqlx::query(
            r#"
            SELECT result_data, expires_at
            FROM result_cache
            WHERE cache_key = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        let expires_at: i64 = row.try_get("expires_at").ok()?;
        if Utc::now().timestamp() >= expires_at {
            let _ = self.delete(key).await;
            return None;
        }

        row.try_get("result_data").ok()
    }

    /// Store consolidated results in cache with TTL.
    pub async fn set(&self, key: &str, value: &str, query: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().timestamp();
        let expires_at = (Utc::now() + self.ttl).timestamp();

        sqlx::query(
            r#"
            INSERT INTO result_cache (cache_key, query_hash, result_data, cached_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                query_hash = excluded.query_hash,
                result_data = excluded.result_data,
                cached_at = excluded.cached_at,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(key)
        .bind(query)
        .bind(value)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM result_cache WHERE cache_key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clean up expired cache entries.
    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let now = Utc::now().timestamp();

        let result = sqlx::query("DELETE FROM result_cache WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let query1 = CacheQuery {
            providers: vec!["modrinth".to_string(), "curseforge".to_string()],
            query: "fabric performance".to_string(),
            filters: serde_json::json!({"version": "1.20.1"}),
        };

        let query2 = CacheQuery {
            providers: vec!["curseforge".to_string(), "modrinth".to_string()],
            query: "fabric performance".to_string(),
            filters: serde_json::json!({"version": "1.20.1"}),
        };

        let key1 = ResultCache::generate_cache_key(&query1);
        let key2 = ResultCache::generate_cache_key(&query2);

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64);
    }
}
