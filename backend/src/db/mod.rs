// Database module for SQLite connections
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;
use tokio::fs;

pub struct DatabasePools {
    pub sessions: SqlitePool,
    pub cache: SqlitePool,
    pub discovery: SqlitePool,
}

impl DatabasePools {
    pub async fn new(data_dir: &str) -> Result<Self, sqlx::Error> {
        // Ensure data directory exists
        fs::create_dir_all(data_dir).await.ok();
        
        // Create database files with WAL mode
        let sessions = Self::create_pool(&format!("{}/sessions.db", data_dir)).await?;
        let cache = Self::create_pool(&format!("{}/cache.db", data_dir)).await?;
        let discovery = Self::create_pool(&format!("{}/discovery.db", data_dir)).await?;
        
        // Run migrations
        Self::run_migrations(&sessions, "migrations/001_create_sessions.sql").await?;
        Self::run_migrations(&cache, "migrations/002_create_cache.sql").await?;
        Self::run_migrations(&discovery, "migrations/003_create_discovery.sql").await?;
        
        Ok(Self {
            sessions,
            cache,
            discovery,
        })
    }
    
    async fn create_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        
        SqlitePool::connect_with(options).await
    }
    
    async fn run_migrations(pool: &SqlitePool, migration_file: &str) -> Result<(), sqlx::Error> {
        if Path::new(migration_file).exists() {
            let sql = tokio::fs::read_to_string(migration_file).await
                .map_err(|e| sqlx::Error::Io(e))?;
            sqlx::query(&sql).execute(pool).await?;
        }
        Ok(())
    }
}
