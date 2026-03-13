use std::{future::Future, pin::Pin};

use thiserror::Error;

/** Type alias for async data operations that return a Future */
pub type DataFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/**
 * DataError
 *
 * Error types that can occur during data layer operations.
 */
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

/**
 * DocumentRepo Trait
 *
 * A trait for implementing document repositories in NestForge.
 * Provides standard CRUD operations for entities.
 *
 * # Type Parameters
 * - `T`: The document/entity type
 * - `Id`: The ID type for the document
 *
 * # Methods
 * - `find_all`: Retrieves all documents
 * - `find_by_id`: Retrieves a single document by ID
 * - `insert`: Creates a new document
 * - `update`: Updates an existing document
 * - `delete`: Removes a document
 */
pub trait DocumentRepo<T>: Send + Sync {
    /** The ID type for the document */
    type Id: Send + Sync + Clone + 'static;

    /**
     * Retrieves all documents of type T.
     */
    fn find_all(&self) -> DataFuture<'_, Result<Vec<T>, DataError>>;

    /**
     * Retrieves a single document by its ID.
     */
    fn find_by_id(&self, id: Self::Id) -> DataFuture<'_, Result<Option<T>, DataError>>;

    /**
     * Inserts a new document.
     */
    fn insert(&self, doc: T) -> DataFuture<'_, Result<T, DataError>>;

    /**
     * Updates an existing document by ID.
     */
    fn update(&self, id: Self::Id, doc: T) -> DataFuture<'_, Result<T, DataError>>;

    /**
     * Deletes a document by ID.
     */
    fn delete(&self, id: Self::Id) -> DataFuture<'_, Result<(), DataError>>;
}

/**
 * CacheStore Trait
 *
 * A trait for implementing cache storage backends.
 * Provides key-value caching operations with optional TTL.
 *
 * # Methods
 * - `get`: Retrieves a value by key
 * - `set`: Stores a value with optional expiration
 * - `delete`: Removes a value by key
 */
pub trait CacheStore: Send + Sync {
    /**
     * Retrieves a cached value by key.
     *
     * Returns None if the key doesn't exist or has expired.
     */
    fn get(&self, key: &str) -> DataFuture<'_, Result<Option<String>, DataError>>;

    /**
     * Stores a value in the cache.
     *
     * # Arguments
     * - `key`: The cache key
     * - `value`: The value to store
     * - `ttl_seconds`: Optional expiration time in seconds
     */
    fn set(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> DataFuture<'_, Result<(), DataError>>;

    /**
     * Removes a value from the cache.
     */
    fn delete(&self, key: &str) -> DataFuture<'_, Result<(), DataError>>;
}
