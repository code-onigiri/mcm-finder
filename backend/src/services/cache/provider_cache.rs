// T043: Provider response cache with SQLite storage and 24h TTL
use chrono::{Duration, Utc};
use sqlx::{Row, SqlitePool};

pub struct ProviderCache {
    pool: SqlitePool,
    ttl: Duration,
}

impl ProviderCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            ttl: Duration::hours(24),
        }
    }

    pub fn with_ttl(pool: SqlitePool, ttl: Duration) -> Self {
        Self { pool, ttl }
    }

    /// Get cached response if it exists and hasn't expired.
    pub async fn get(&self, key: &str) -> Option<String> {
        let row = sqlx::query(
            r#"
            SELECT response_json, expires_at
            FROM provider_responses
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

        row.try_get("response_json").ok()
    }

    /// Store response in cache with TTL.
    pub async fn set(&self, key: &str, value: &str, provider: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().timestamp();
        let expires_at = (Utc::now() + self.ttl).timestamp();

        sqlx::query(
            r#"
            INSERT INTO provider_responses (cache_key, provider, response_json, cached_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                provider = excluded.provider,
                response_json = excluded.response_json,
                cached_at = excluded.cached_at,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(key)
        .bind(provider)
        .bind(value)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM provider_responses WHERE cache_key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clean up expired cache entries.
    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let now = Utc::now().timestamp();

        let result = sqlx::query("DELETE FROM provider_responses WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
