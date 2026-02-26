use std::{future::Future, pin::Pin};

use thiserror::Error;

pub type DataFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Query failed: {0}")]
    Query(String),
    #[error("Serialization failed: {0}")]
    Serialization(String),
    #[error("Not found")]
    NotFound,
}

pub trait DocumentRepo<T>: Send + Sync {
    type Id: Send + Sync + Clone + 'static;
    fn find_all(&self) -> DataFuture<'_, Result<Vec<T>, DataError>>;
    fn find_by_id(&self, id: Self::Id) -> DataFuture<'_, Result<Option<T>, DataError>>;
    fn insert(&self, doc: T) -> DataFuture<'_, Result<T, DataError>>;
    fn update(&self, id: Self::Id, doc: T) -> DataFuture<'_, Result<T, DataError>>;
    fn delete(&self, id: Self::Id) -> DataFuture<'_, Result<(), DataError>>;
}

pub trait CacheStore: Send + Sync {
    fn get(&self, key: &str) -> DataFuture<'_, Result<Option<String>, DataError>>;
    fn set(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> DataFuture<'_, Result<(), DataError>>;
    fn delete(&self, key: &str) -> DataFuture<'_, Result<(), DataError>>;
}
