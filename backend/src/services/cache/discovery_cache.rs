use crate::models::DiscoveryEvidenceItem;
use chrono::{Duration, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub struct DiscoveryCache {
    pool: SqlitePool,
    ttl: Duration,
}

impl DiscoveryCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            ttl: Duration::days(30),
        }
    }

    pub async fn get(&self, mod_profile_id: &Uuid) -> Option<Vec<DiscoveryEvidenceItem>> {
        self.ensure_schema().await.ok()?;

        let row = sqlx::query(
            r#"
            SELECT evidence_json, expires_at
            FROM discovery_cache
            WHERE mod_profile_id = ?
            "#,
        )
        .bind(mod_profile_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        let expires_at: i64 = row.try_get("expires_at").ok()?;
        if Utc::now().timestamp() >= expires_at {
            let _ = self.delete(mod_profile_id).await;
            return None;
        }

        let evidence_json: String = row.try_get("evidence_json").ok()?;
        serde_json::from_str(&evidence_json).ok()
    }

    pub async fn set(
        &self,
        mod_profile_id: &Uuid,
        evidence: &[DiscoveryEvidenceItem],
    ) -> Result<(), sqlx::Error> {
        self.ensure_schema().await?;

        let evidence_json = serde_json::to_string(evidence)
            .map_err(|err| sqlx::Error::Protocol(err.to_string()))?;
        let now = Utc::now().timestamp();
        let expires_at = (Utc::now() + self.ttl).timestamp();

        sqlx::query(
            r#"
            INSERT INTO discovery_cache (mod_profile_id, evidence_json, cached_at, expires_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(mod_profile_id) DO UPDATE SET
                evidence_json = excluded.evidence_json,
                cached_at = excluded.cached_at,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(mod_profile_id.to_string())
        .bind(evidence_json)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, mod_profile_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM discovery_cache WHERE mod_profile_id = ?")
            .bind(mod_profile_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn ensure_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS discovery_cache (
                mod_profile_id TEXT PRIMARY KEY,
                evidence_json TEXT NOT NULL,
                cached_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
