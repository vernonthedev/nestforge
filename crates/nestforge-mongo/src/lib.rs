use std::{collections::HashMap, sync::{Arc, RwLock}};

use nestforge_data::{DataError, DocumentRepo};

#[derive(Clone)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
}

impl MongoConfig {
    pub fn new(uri: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            database: database.into(),
        }
    }
}

#[derive(Clone)]
pub struct InMemoryMongoRepo<T> {
    docs: Arc<RwLock<HashMap<String, T>>>,
}

impl<T> InMemoryMongoRepo<T> {
    pub fn new() -> Self {
        Self {
            docs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> Default for InMemoryMongoRepo<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DocumentRepo<T> for InMemoryMongoRepo<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Id = String;

    fn find_all(&self) -> nestforge_data::DataFuture<'_, Result<Vec<T>, DataError>> {
        let docs = Arc::clone(&self.docs);
        Box::pin(async move {
            let map = docs
                .read()
                .map_err(|_| DataError::Query("in-memory mongo lock poisoned".to_string()))?;
            Ok(map.values().cloned().collect())
        })
    }

    fn find_by_id(&self, id: Self::Id) -> nestforge_data::DataFuture<'_, Result<Option<T>, DataError>> {
        let docs = Arc::clone(&self.docs);
        Box::pin(async move {
            let map = docs
                .read()
                .map_err(|_| DataError::Query("in-memory mongo lock poisoned".to_string()))?;
            Ok(map.get(&id).cloned())
        })
    }

    fn insert(&self, doc: T) -> nestforge_data::DataFuture<'_, Result<T, DataError>> {
        let docs = Arc::clone(&self.docs);
        Box::pin(async move {
            let id = format!("doc_{}", docs.read().map_err(|_| DataError::Query("lock".to_string()))?.len() + 1);
            docs.write()
                .map_err(|_| DataError::Query("in-memory mongo lock poisoned".to_string()))?
                .insert(id, doc.clone());
            Ok(doc)
        })
    }

    fn update(&self, id: Self::Id, doc: T) -> nestforge_data::DataFuture<'_, Result<T, DataError>> {
        let docs = Arc::clone(&self.docs);
        Box::pin(async move {
            docs.write()
                .map_err(|_| DataError::Query("in-memory mongo lock poisoned".to_string()))?
                .insert(id, doc.clone());
            Ok(doc)
        })
    }

    fn delete(&self, id: Self::Id) -> nestforge_data::DataFuture<'_, Result<(), DataError>> {
        let docs = Arc::clone(&self.docs);
        Box::pin(async move {
            docs.write()
                .map_err(|_| DataError::Query("in-memory mongo lock poisoned".to_string()))?
                .remove(&id);
            Ok(())
        })
    }
}
