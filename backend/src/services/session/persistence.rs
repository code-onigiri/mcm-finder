use crate::models::SearchSessionSummary;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionPersistence {
    pool: SqlitePool,
}

impl SessionPersistence {
    pub async fn new(pool: SqlitePool) -> Result<Self, sqlx::Error> {
        let persistence = Self { pool };
        persistence.ensure_schema().await?;
        Ok(persistence)
    }

    pub async fn save(&self, session: &SearchSessionSummary) -> Result<(), sqlx::Error> {
        let payload =
            serde_json::to_string(session).map_err(|err| sqlx::Error::Protocol(err.to_string()))?;
        let expires_at = session.expires_at.map(|timestamp| timestamp.timestamp());

        sqlx::query(
            r#"
            INSERT INTO saved_sessions (
                id,
                user_identifier,
                payload_json,
                created_at,
                updated_at,
                expires_at
            )
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                user_identifier = excluded.user_identifier,
                payload_json = excluded.payload_json,
                updated_at = excluded.updated_at,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.user_identifier.clone())
        .bind(payload)
        .bind(session.created_at.timestamp())
        .bind(session.updated_at.timestamp())
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<SearchSessionSummary>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT payload_json, expires_at
            FROM saved_sessions
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let expires_at: Option<i64> = row.try_get("expires_at")?;
        if let Some(expires_at) = expires_at {
            if Utc::now().timestamp() >= expires_at {
                let _ = self.delete(id).await;
                return Ok(None);
            }
        }

        let payload_json: String = row.try_get("payload_json")?;
        let session = serde_json::from_str::<SearchSessionSummary>(&payload_json)
            .map_err(|err| sqlx::Error::Protocol(err.to_string()))?;
        Ok(Some(session))
    }

    pub async fn list_recent(
        &self,
        user_identifier: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchSessionSummary>, sqlx::Error> {
        self.cleanup_expired().await?;

        let rows = if let Some(user_identifier) = user_identifier {
            sqlx::query(
                r#"
                SELECT payload_json
                FROM saved_sessions
                WHERE user_identifier = ?
                ORDER BY updated_at DESC
                LIMIT ?
                "#,
            )
            .bind(user_identifier)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT payload_json
                FROM saved_sessions
                ORDER BY updated_at DESC
                LIMIT ?
                "#,
            )
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let mut sessions = Vec::new();
        for row in rows {
            let payload_json: String = row.try_get("payload_json")?;
            if let Ok(session) = serde_json::from_str::<SearchSessionSummary>(&payload_json) {
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    pub async fn cleanup_expired(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM saved_sessions
            WHERE expires_at IS NOT NULL AND expires_at <= ?
            "#,
        )
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saved_sessions WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn ensure_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saved_sessions (
                id TEXT PRIMARY KEY,
                user_identifier TEXT,
                payload_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
