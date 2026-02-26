use std::{collections::HashMap, time::Duration};

use sqlx::{
    any::{AnyPoolOptions, AnyRow},
    Any, AnyPool, FromRow, Transaction,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
}

impl DbConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(10),
        }
    }

    pub fn postgres_local(database: &str) -> Self {
        Self::new(format!("postgres://postgres:postgres@localhost/{database}"))
    }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Invalid database configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("Failed to connect to database `{url}`: {source}")]
    Connect {
        url: String,
        #[source]
        source: sqlx::Error,
    },
    #[error("Named connection `{name}` was not configured")]
    NamedConnectionNotFound { name: String },
    #[error("Database query failed: {0}")]
    Query(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct Db {
    primary: AnyPool,
    named: HashMap<String, AnyPool>,
}

impl Db {
    pub async fn connect(config: DbConfig) -> Result<Self, DbError> {
        let primary = connect_pool(&config).await?;
        Ok(Self {
            primary,
            named: HashMap::new(),
        })
    }

    pub fn connect_lazy(config: DbConfig) -> Result<Self, DbError> {
        let primary = connect_pool_lazy(&config)?;
        Ok(Self {
            primary,
            named: HashMap::new(),
        })
    }

    pub async fn connect_many<I>(primary: DbConfig, named: I) -> Result<Self, DbError>
    where
        I: IntoIterator<Item = (String, DbConfig)>,
    {
        let primary_pool = connect_pool(&primary).await?;
        let mut named_pools = HashMap::new();

        for (name, config) in named {
            let pool = connect_pool(&config).await?;
            named_pools.insert(name, pool);
        }

        Ok(Self {
            primary: primary_pool,
            named: named_pools,
        })
    }

    pub fn connect_many_lazy<I>(primary: DbConfig, named: I) -> Result<Self, DbError>
    where
        I: IntoIterator<Item = (String, DbConfig)>,
    {
        let primary_pool = connect_pool_lazy(&primary)?;
        let mut named_pools = HashMap::new();

        for (name, config) in named {
            let pool = connect_pool_lazy(&config)?;
            named_pools.insert(name, pool);
        }

        Ok(Self {
            primary: primary_pool,
            named: named_pools,
        })
    }

    pub fn pool(&self) -> &AnyPool {
        &self.primary
    }

    pub fn pool_named(&self, name: &str) -> Result<&AnyPool, DbError> {
        self.named
            .get(name)
            .ok_or_else(|| DbError::NamedConnectionNotFound {
                name: name.to_string(),
            })
    }

    pub async fn execute(&self, sql: &str) -> Result<u64, DbError> {
        let result = sqlx::query::<Any>(sql).execute(&self.primary).await?;
        Ok(result.rows_affected())
    }

    pub async fn execute_named(&self, name: &str, sql: &str) -> Result<u64, DbError> {
        let pool = self.pool_named(name)?;
        let result = sqlx::query::<Any>(sql).execute(pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn fetch_all<T>(&self, sql: &str) -> Result<Vec<T>, DbError>
    where
        for<'r> T: FromRow<'r, AnyRow> + Send + Unpin,
    {
        let rows = sqlx::query_as::<Any, T>(sql)
            .fetch_all(&self.primary)
            .await?;
        Ok(rows)
    }

    pub async fn fetch_all_named<T>(&self, name: &str, sql: &str) -> Result<Vec<T>, DbError>
    where
        for<'r> T: FromRow<'r, AnyRow> + Send + Unpin,
    {
        let pool = self.pool_named(name)?;
        let rows = sqlx::query_as::<Any, T>(sql).fetch_all(pool).await?;
        Ok(rows)
    }

    pub async fn begin(&self) -> Result<DbTransaction, DbError> {
        let tx = self.primary.begin().await?;
        Ok(DbTransaction { tx })
    }

    pub async fn begin_named(&self, name: &str) -> Result<DbTransaction, DbError> {
        let pool = self.pool_named(name)?;
        let tx = pool.begin().await?;
        Ok(DbTransaction { tx })
    }
}

pub struct DbTransaction {
    tx: Transaction<'static, Any>,
}

impl DbTransaction {
    pub async fn execute(&mut self, sql: &str) -> Result<u64, DbError> {
        let result = sqlx::query::<Any>(sql).execute(&mut *self.tx).await?;
        Ok(result.rows_affected())
    }

    pub async fn fetch_all<T>(&mut self, sql: &str) -> Result<Vec<T>, DbError>
    where
        for<'r> T: FromRow<'r, AnyRow> + Send + Unpin,
    {
        let rows = sqlx::query_as::<Any, T>(sql)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows)
    }

    pub async fn commit(self) -> Result<(), DbError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), DbError> {
        self.tx.rollback().await?;
        Ok(())
    }
}

async fn connect_pool(config: &DbConfig) -> Result<AnyPool, DbError> {
    if config.url.trim().is_empty() {
        return Err(DbError::InvalidConfig("url cannot be empty"));
    }

    sqlx::any::install_default_drivers();

    AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect(&config.url)
        .await
        .map_err(|source| DbError::Connect {
            url: config.url.clone(),
            source,
        })
}

fn connect_pool_lazy(config: &DbConfig) -> Result<AnyPool, DbError> {
    if config.url.trim().is_empty() {
        return Err(DbError::InvalidConfig("url cannot be empty"));
    }

    sqlx::any::install_default_drivers();

    AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect_lazy(&config.url)
        .map_err(|source| DbError::Connect {
            url: config.url.clone(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_empty_url_configuration() {
        let config = DbConfig::new("");
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let err = rt
            .block_on(connect_pool(&config))
            .expect_err("config should fail");

        assert!(matches!(err, DbError::InvalidConfig(_)));
    }

    #[tokio::test]
    async fn returns_named_connection_error_for_missing_pool() {
        let db = Db {
            primary: AnyPoolOptions::new()
                .connect_lazy("postgres://postgres:postgres@localhost/postgres")
                .expect("lazy pool"),
            named: HashMap::new(),
        };

        let err = db.pool_named("analytics").expect_err("missing pool should fail");
        assert!(matches!(err, DbError::NamedConnectionNotFound { .. }));
    }

    #[tokio::test]
    async fn creates_lazy_db_for_sync_module_registration() {
        let db = Db::connect_lazy(DbConfig::postgres_local("postgres"));
        assert!(db.is_ok(), "lazy db creation should succeed");
    }
}
