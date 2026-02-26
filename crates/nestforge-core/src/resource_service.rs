use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{Identifiable, InMemoryStore};

#[derive(Clone)]
pub struct ResourceService<T>
where
    T: Identifiable + Clone,
{
    store: InMemoryStore<T>,
}

impl<T> Default for ResourceService<T>
where
    T: Identifiable + Clone + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Failed to serialize dto: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("DTO must serialize into a JSON object")]
    DtoNotObject,
    #[error("Entity must serialize into a JSON object")]
    EntityNotObject,
}

impl<T> ResourceService<T>
where
    T: Identifiable + Clone + Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            store: InMemoryStore::new(),
        }
    }

    pub fn with_seed(seed: Vec<T>) -> Self {
        Self {
            store: InMemoryStore::with_seed(seed),
        }
    }

    pub fn find_all(&self) -> Vec<T> {
        self.store.find_all()
    }

    pub fn all(&self) -> Vec<T> {
        self.find_all()
    }

    pub fn find_by_id(&self, id: u64) -> Option<T> {
        self.store.find_by_id(id)
    }

    pub fn get(&self, id: u64) -> Option<T> {
        self.find_by_id(id)
    }

    pub fn count(&self) -> usize {
        self.store.count()
    }

    pub fn exists(&self, id: u64) -> bool {
        self.find_by_id(id).is_some()
    }

    pub fn create_from<D>(&self, dto: D) -> Result<T, ResourceError>
    where
        D: Serialize,
    {
        let mut dto_value = serde_json::to_value(dto)?;
        let Some(map) = dto_value.as_object_mut() else {
            return Err(ResourceError::DtoNotObject);
        };

        map.insert("id".to_string(), Value::from(0_u64));
        let entity = serde_json::from_value::<T>(dto_value)?;
        Ok(self.store.create(entity))
    }

    pub fn create<D>(&self, dto: D) -> Result<T, ResourceError>
    where
        D: Serialize,
    {
        self.create_from(dto)
    }

    pub fn update_from<D>(&self, id: u64, dto: D) -> Result<Option<T>, ResourceError>
    where
        D: Serialize,
    {
        let dto_value = serde_json::to_value(dto)?;
        let Some(dto_map) = dto_value.as_object() else {
            return Err(ResourceError::DtoNotObject);
        };

        let mut dto_map = dto_map.clone();
        dto_map.remove("id");

        let Some(existing) = self.store.find_by_id(id) else {
            return Ok(None);
        };

        let mut entity_value = serde_json::to_value(existing)?;
        let Some(entity_map) = entity_value.as_object_mut() else {
            return Err(ResourceError::EntityNotObject);
        };

        for (key, value) in dto_map {
            if !value.is_null() {
                entity_map.insert(key, value);
            }
        }

        let merged = serde_json::from_value::<T>(entity_value)?;
        let updated = self.store.update_by_id(id, |entity| {
            *entity = merged.clone();
        });

        Ok(updated)
    }

    pub fn update<D>(&self, id: u64, dto: D) -> Result<Option<T>, ResourceError>
    where
        D: Serialize,
    {
        self.update_from(id, dto)
    }

    pub fn replace_from<D>(&self, id: u64, dto: D) -> Result<Option<T>, ResourceError>
    where
        D: Serialize,
    {
        let mut dto_value = serde_json::to_value(dto)?;
        let Some(map) = dto_value.as_object_mut() else {
            return Err(ResourceError::DtoNotObject);
        };

        map.insert("id".to_string(), Value::from(id));
        let replacement = serde_json::from_value::<T>(dto_value)?;
        Ok(self.store.replace_by_id(id, replacement))
    }

    pub fn replace<D>(&self, id: u64, dto: D) -> Result<Option<T>, ResourceError>
    where
        D: Serialize,
    {
        self.replace_from(id, dto)
    }

    pub fn delete_by_id(&self, id: u64) -> Option<T> {
        self.store.delete_by_id(id)
    }

    pub fn delete(&self, id: u64) -> Option<T> {
        self.delete_by_id(id)
    }
}
