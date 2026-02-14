pub mod persistence;

use crate::models::SearchSessionSummary;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::persistence::SessionPersistence;

#[derive(Clone, Default)]
pub struct SessionService {
    persistence: Option<Arc<SessionPersistence>>,
    in_memory: Arc<RwLock<HashMap<Uuid, SearchSessionSummary>>>,
}

impl SessionService {
    pub fn new(persistence: Option<Arc<SessionPersistence>>) -> Self {
        Self {
            persistence,
            in_memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save(
        &self,
        session: SearchSessionSummary,
    ) -> Result<SearchSessionSummary, sqlx::Error> {
        self.in_memory
            .write()
            .await
            .insert(session.id, session.clone());
        if let Some(persistence) = &self.persistence {
            persistence.save(&session).await?;
        }
        Ok(session)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<SearchSessionSummary>, sqlx::Error> {
        if let Some(session) = self.in_memory.read().await.get(&id).cloned() {
            return Ok(Some(session));
        }

        if let Some(persistence) = &self.persistence {
            if let Some(session) = persistence.get(id).await? {
                self.in_memory.write().await.insert(id, session.clone());
                return Ok(Some(session));
            }
        }

        Ok(None)
    }

    pub async fn list_recent(
        &self,
        user_identifier: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchSessionSummary>, sqlx::Error> {
        if let Some(persistence) = &self.persistence {
            let sessions = persistence.list_recent(user_identifier, limit).await?;
            {
                let mut memory = self.in_memory.write().await;
                for session in &sessions {
                    memory.insert(session.id, session.clone());
                }
            }
            return Ok(sessions);
        }

        let mut sessions: Vec<SearchSessionSummary> =
            self.in_memory.read().await.values().cloned().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        if sessions.len() > limit {
            sessions.truncate(limit);
        }
        Ok(sessions)
    }
}
