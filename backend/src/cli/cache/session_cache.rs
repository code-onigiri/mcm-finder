use chrono::{Duration, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::path::Path;

const CACHE_VERSION: &str = "1";
const DEFAULT_TTL_DAYS: i64 = 7;

#[derive(Debug, Clone)]
pub struct CacheMetadata {
    pub cached_at: i64,
    pub expires_at: i64,
    pub cache_version: String,
}

pub struct CliSessionCache {
    pool: SqlitePool,
    ttl: Duration,
}

impl CliSessionCache {
    pub async fn new(path: &str) -> Result<Self, sqlx::Error> {
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cli_session_cache (
                cache_key TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                cached_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                cache_version TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            ttl: Duration::days(DEFAULT_TTL_DAYS),
        })
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        key: &str,
        force_refresh: bool,
    ) -> Result<Option<(T, CacheMetadata)>, sqlx::Error> {
        if force_refresh {
            return Ok(None);
        }

        let row = sqlx::query(
            r#"
            SELECT payload_json, cached_at, expires_at, cache_version
            FROM cli_session_cache
            WHERE cache_key = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let cached_at: i64 = row.try_get("cached_at")?;
        let expires_at: i64 = row.try_get("expires_at")?;
        let cache_version: String = row.try_get("cache_version")?;

        // T068: freshness checks for expiry and cache version.
        if Utc::now().timestamp() >= expires_at || cache_version != CACHE_VERSION {
            self.delete(key).await?;
            return Ok(None);
        }

        let payload_json: String = row.try_get("payload_json")?;
        let parsed = match serde_json::from_str::<T>(&payload_json) {
            Ok(value) => value,
            Err(_) => {
                self.delete(key).await?;
                return Ok(None);
            }
        };

        Ok(Some((
            parsed,
            CacheMetadata {
                cached_at,
                expires_at,
                cache_version,
            },
        )))
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<CacheMetadata, sqlx::Error> {
        let payload_json =
            serde_json::to_string(value).map_err(|err| sqlx::Error::Protocol(err.to_string()))?;

        let cached_at = Utc::now().timestamp();
        let expires_at = (Utc::now() + self.ttl).timestamp();

        sqlx::query(
            r#"
            INSERT INTO cli_session_cache (cache_key, payload_json, cached_at, expires_at, cache_version)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                payload_json = excluded.payload_json,
                cached_at = excluded.cached_at,
                expires_at = excluded.expires_at,
                cache_version = excluded.cache_version
            "#,
        )
        .bind(key)
        .bind(payload_json)
        .bind(cached_at)
        .bind(expires_at)
        .bind(CACHE_VERSION)
        .execute(&self.pool)
        .await?;

        Ok(CacheMetadata {
            cached_at,
            expires_at,
            cache_version: CACHE_VERSION.to_string(),
        })
    }

    async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM cli_session_cache WHERE cache_key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
