use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
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
    overrides: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    request_factories: Arc<
        RwLock<HashMap<TypeId, Arc<RequestFactoryFn>>>,
    >,
    transient_factories: Arc<
        RwLock<HashMap<TypeId, Arc<TransientFactoryFn>>>,
    >,
    names: Arc<RwLock<HashSet<&'static str>>>,
}

type RequestFactoryValue = Arc<dyn Any + Send + Sync>;
type RequestFactoryFn =
    dyn Fn(&Container) -> anyhow::Result<RequestFactoryValue> + Send + Sync + 'static;
type TransientFactoryFn =
    dyn Fn(&Container) -> anyhow::Result<RequestFactoryValue> + Send + Sync + 'static;

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
    #[error("Request-scoped factory failed for {type_name}: {message}")]
    RequestFactoryFailed {
        type_name: &'static str,
        message: String,
    },
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

    pub fn scoped(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            request_factories: Arc::clone(&self.request_factories),
            transient_factories: Arc::clone(&self.transient_factories),
            names: Arc::clone(&self.names),
        }
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

        if map.contains_key(&type_id) {
            return Err(ContainerError::TypeAlreadyRegistered {
                type_name: std::any::type_name::<T>(),
            });
        }

        map.insert(type_id, Arc::new(value));
        self.names
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?
            .insert(std::any::type_name::<T>());
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
        self.names
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?
            .insert(std::any::type_name::<T>());
        Ok(())
    }

    pub fn override_value<T>(&self, value: T) -> Result<(), ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let mut overrides = self
            .overrides
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?;

        overrides.insert(TypeId::of::<T>(), Arc::new(value));
        self.names
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?
            .insert(std::any::type_name::<T>());
        Ok(())
    }

    pub fn is_type_registered_name(&self, type_name: &'static str) -> Result<bool, ContainerError> {
        let names = self
            .names
            .read()
            .map_err(|_| ContainerError::ReadLockPoisoned)?;
        Ok(names.contains(type_name))
    }

    pub fn register_request_factory<T, F>(&self, factory: F) -> Result<(), ContainerError>
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> anyhow::Result<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut factories = self
            .request_factories
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?;

        if factories.contains_key(&type_id) {
            return Err(ContainerError::TypeAlreadyRegistered {
                type_name: std::any::type_name::<T>(),
            });
        }

        factories.insert(
            type_id,
            Arc::new(move |container| Ok(Arc::new(factory(container)?) as RequestFactoryValue)),
        );
        self.names
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?
            .insert(std::any::type_name::<T>());
        Ok(())
    }

    pub fn register_transient_factory<T, F>(&self, factory: F) -> Result<(), ContainerError>
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> anyhow::Result<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut factories = self
            .transient_factories
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?;

        if factories.contains_key(&type_id) {
            return Err(ContainerError::TypeAlreadyRegistered {
                type_name: std::any::type_name::<T>(),
            });
        }

        factories.insert(
            type_id,
            Arc::new(move |container| Ok(Arc::new(factory(container)?) as RequestFactoryValue)),
        );
        self.names
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?
            .insert(std::any::type_name::<T>());
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
        if let Some(value) = self.resolve_from_map::<T>(&self.overrides)? {
            return Ok(value);
        }

        if let Some(value) = self.resolve_from_map::<T>(&self.inner)? {
            return Ok(value);
        }

        if let Some(value) = self.resolve_from_request_factory::<T>()? {
            return Ok(value);
        }

        if let Some(value) = self.resolve_from_transient_factory::<T>()? {
            return Ok(value);
        }

        Err(ContainerError::TypeNotRegistered {
            type_name: std::any::type_name::<T>(),
        })
    }

    pub fn resolve_in_module<T>(&self, module_name: &'static str) -> Result<Arc<T>, ContainerError>
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

    fn resolve_from_map<T>(
        &self,
        map: &Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    ) -> Result<Option<Arc<T>>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let map = map.read().map_err(|_| ContainerError::ReadLockPoisoned)?;
        let Some(value) = map.get(&TypeId::of::<T>()).cloned() else {
            return Ok(None);
        };

        let value = value.downcast::<T>().map_err(|_| ContainerError::DowncastFailed {
            type_name: std::any::type_name::<T>(),
        })?;

        Ok(Some(value))
    }

    fn resolve_from_request_factory<T>(&self) -> Result<Option<Arc<T>>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let factory = {
            let factories = self
                .request_factories
                .read()
                .map_err(|_| ContainerError::ReadLockPoisoned)?;
            factories.get(&TypeId::of::<T>()).cloned()
        };

        let Some(factory) = factory else {
            return Ok(None);
        };

        let value = factory(self).map_err(|err| ContainerError::RequestFactoryFailed {
            type_name: std::any::type_name::<T>(),
            message: err.to_string(),
        })?;
        let typed = value.downcast::<T>().map_err(|_| ContainerError::DowncastFailed {
            type_name: std::any::type_name::<T>(),
        })?;

        self.overrides
            .write()
            .map_err(|_| ContainerError::WriteLockPoisoned)?
            .insert(TypeId::of::<T>(), typed.clone() as RequestFactoryValue);

        Ok(Some(typed))
    }

    fn resolve_from_transient_factory<T>(&self) -> Result<Option<Arc<T>>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        let factory = {
            let factories = self
                .transient_factories
                .read()
                .map_err(|_| ContainerError::ReadLockPoisoned)?;
            factories.get(&TypeId::of::<T>()).cloned()
        };

        let Some(factory) = factory else {
            return Ok(None);
        };

        let value = factory(self).map_err(|err| ContainerError::RequestFactoryFailed {
            type_name: std::any::type_name::<T>(),
            message: err.to_string(),
        })?;
        let typed = value.downcast::<T>().map_err(|_| ContainerError::DowncastFailed {
            type_name: std::any::type_name::<T>(),
        })?;

        Ok(Some(typed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct AppConfig {
        app_name: &'static str,
    }

    #[test]
    fn override_value_takes_precedence_over_registered_value() {
        let container = Container::new();

        container
            .register(AppConfig {
                app_name: "default",
            })
            .expect("register should succeed");
        container
            .override_value(AppConfig { app_name: "test" })
            .expect("override should succeed");

        let config = container
            .resolve::<AppConfig>()
            .expect("config should resolve");
        assert_eq!(config.app_name, "test");
    }

    #[derive(Clone)]
    struct RequestId(String);

    struct RequestGreeting(String);
    struct TransientCounter(usize);

    #[test]
    fn scoped_container_resolves_request_factory_without_leaking_to_parent() {
        let container = Container::new();
        container
            .register_request_factory::<RequestGreeting, _>(|scoped| {
                let request_id = scoped.resolve::<RequestId>()?;
                Ok(RequestGreeting(format!("hello {}", request_id.0)))
            })
            .expect("request factory should register");

        let scoped = container.scoped();
        scoped
            .override_value(RequestId("req-1".to_string()))
            .expect("request id should override");

        let greeting = scoped
            .resolve::<RequestGreeting>()
            .expect("request greeting should resolve");

        assert_eq!(greeting.0, "hello req-1");
        assert!(container.resolve::<RequestGreeting>().is_err());
    }

    #[test]
    fn transient_factory_creates_new_instances_per_resolve() {
        let container = Container::new();
        let counter = Arc::new(RwLock::new(0usize));
        let counter_for_factory = Arc::clone(&counter);

        container
            .register_transient_factory::<TransientCounter, _>(move |_| {
                let mut count = counter_for_factory
                    .write()
                    .expect("counter should be writable");
                *count += 1;
                Ok(TransientCounter(*count))
            })
            .expect("transient factory should register");

        let first = container
            .resolve::<TransientCounter>()
            .expect("first transient should resolve");
        let second = container
            .resolve::<TransientCounter>()
            .expect("second transient should resolve");

        assert_eq!(first.0, 1);
        assert_eq!(second.0, 2);
    }
}
