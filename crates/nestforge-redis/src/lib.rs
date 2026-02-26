use std::{collections::HashMap, sync::{Arc, RwLock}};

use nestforge_data::{CacheStore, DataError};

#[derive(Clone)]
pub struct RedisConfig {
    pub url: String,
}

impl RedisConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

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
        _ttl_seconds: Option<u64>,
    ) -> nestforge_data::DataFuture<'_, Result<(), DataError>> {
        let key = key.to_string();
        let value = value.to_string();
        let map = Arc::clone(&self.values);
        Box::pin(async move {
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
