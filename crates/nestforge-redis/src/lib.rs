use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nestforge_data::{CacheStore, DataError};

/**
 * RedisConfig
 *
 * Configuration for Redis connection.
 */
#[derive(Clone)]
pub struct RedisConfig {
    /** The Redis connection URL */
    pub url: String,
}

impl RedisConfig {
    /**
     * Creates a new RedisConfig with the given URL.
     */
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/**
 * InMemoryRedisStore
 *
 * An in-memory implementation of CacheStore for development/testing.
 * Provides the same interface as Redis but stores data in memory.
 *
 * Note: Does not support TTL (time-to-live) expiration.
 */
#[derive(Clone, Default)]
pub struct InMemoryRedisStore {
    values: Arc<RwLock<HashMap<String, String>>>,
}

impl CacheStore for InMemoryRedisStore {
    fn get(&self, key: &str) -> nestforge_data::DataFuture<'_, Result<Option<String>, DataError>> {
        let key = key.to_string();
        let map = Arc::clone(&self.values);
        Box::pin(async move {
            let map = map
                .read()
                .map_err(|_| DataError::Query("in-memory redis lock poisoned".to_string()))?;
            Ok(map.get(&key).cloned())
        })
    }

    fn set(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> nestforge_data::DataFuture<'_, Result<(), DataError>> {
        let key = key.to_string();
        let value = value.to_string();
        let map = Arc::clone(&self.values);
        Box::pin(async move {
            if ttl_seconds.is_some() {
                return Err(DataError::Query(
                    "in-memory redis store does not support ttl".to_string(),
                ));
            }
            map.write()
                .map_err(|_| DataError::Query("in-memory redis lock poisoned".to_string()))?
                .insert(key, value);
            Ok(())
        })
    }

    fn delete(&self, key: &str) -> nestforge_data::DataFuture<'_, Result<(), DataError>> {
        let key = key.to_string();
        let map = Arc::clone(&self.values);
        Box::pin(async move {
            map.write()
                .map_err(|_| DataError::Query("in-memory redis lock poisoned".to_string()))?
                .remove(&key);
            Ok(())
        })
    }
}
