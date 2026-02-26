use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use thiserror::Error;

/**
* Container = our tiny dependency injection store (v1).
* 
* What it does:
* - stores values by type (TypeId).
* - lets us register services/config once.
* - lets us resolve them later.
* 
* Why Arc?
* - so multiple parts of the app can share the same service safely.
* 
* Why RwLock?
* - allows safe reads/writes across threads.
* - write when registering.
* - read when resolving.
*/

#[derive(Clone, Default)]
pub struct Container {
    inner: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Container write lock poisoned")]
    WriteLockPoisoned,
    #[error("Container read lock poisoned")]
    ReadLockPoisoned,
    #[error("Type already registered: {type_name}")]
    TypeAlreadyRegistered { type_name: &'static str },
    #[error("Type not registered: {type_name}")]
    TypeNotRegistered { type_name: &'static str },
    #[error("Failed to downcast resolved value: {type_name}")]
    DowncastFailed { type_name: &'static str },
    #[error("Type not registered: {type_name} (required by module `{module_name}`)")]
    TypeNotRegisteredInModule {
        type_name: &'static str,
        module_name: &'static str,
    },
}

impl Container {
    /**
    * Nice helper constructor.
    * Same as Default, just cleaner to read in app code.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
    * Register a value/service into the container.
    * 
    * Example:
    * container.register(AppConfig { ... })?;
    * 
    * Rules:
    * - T must be thread-safe (Send + Sync).
    * - T must be 'static because we store it for the app lifetime.
    */
    pub fn register<T>(&self, value: T) -> Result<(), ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let mut map = self
            .inner
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?;

        let type_id = TypeId::of::<T>();

        /* Prevent accidental duplicate registration of the same type. */
        if map.contains_key(&type_id) {
            return Err(ContainerError::TypeAlreadyRegistered {
                type_name: std::any::type_name::<T>(),
            });
        }

         /* Store as Arc<dyn Any> so we can keep different types in one map. */
        map.insert(type_id, Arc::new(value));
        Ok(())
    }

    pub fn replace<T>(&self, value: T) -> Result<(), ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let mut map = self
            .inner
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?;

        map.insert(TypeId::of::<T>(), Arc::new(value));
        Ok(())
    }

    /**
    * Resolve (get back) a registered value/service by type.
    * 
    * Example:
    * let config = container.resolve::<AppConfig>()?;
    * 
    * Returns Arc<T> so the caller can clone/share it cheaply.
    */
    pub fn resolve<T>(&self) -> Result<Arc<T>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let map = self
            .inner
            .read()
            .map_err(|_| ContainerError::ReadLockPoisoned)?;

        let value = map
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ContainerError::TypeNotRegistered {
                type_name: std::any::type_name::<T>(),
            })?
            .clone();

        /*
        * We stored the value as dyn Any, so now we downcast it back to the real type T.
        * If downcast fails, the type in the map doesn’t match what we asked for.
        */
        value.downcast::<T>()
            .map_err(|_| ContainerError::DowncastFailed {
                type_name: std::any::type_name::<T>(),
            })
    }

    pub fn resolve_in_module<T>(
        &self,
        module_name: &'static str,
    ) -> Result<Arc<T>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        self.resolve::<T>().map_err(|err| match err {
            ContainerError::TypeNotRegistered { type_name } => {
                ContainerError::TypeNotRegisteredInModule {
                    type_name,
                    module_name,
                }
            }
            other => other,
        })
    }
}
