use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use thiserror::Error;

/// The Dependency Injection (DI) Container.
///
/// This is the core registry for all providers, services, and configuration in a NestForge application.
/// It mimics the behavior of the NestJS container but is adapted for Rust's ownership and thread-safety models.
///
/// ### Core Features
/// - **Singleton Registry:** By default, registered services are singletons (Arc<T>).
/// - **Thread Safety:** Uses `RwLock` to allow concurrent reads (resolving) and exclusive writes (registering).
/// - **Type-Based Resolution:** Services are stored and retrieved by their `TypeId`.
/// - **Scoped & Transient:** Supports request-scoped and transient factories for more complex lifecycles.
#[derive(Clone, Default)]
pub struct Container {
    /*
    We use Arc<RwLock<...>> because the container itself is cloned and shared across every request.
    The inner HashMap holds the singleton instances as `Arc<dyn Any>`.
    */
    inner: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,

    /*
    Overrides are checked before the main registry. This is primarily used for testing
    or for request-scoped sub-containers that need to shadow parent providers.
    */
    overrides: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,

    request_factories: Arc<RwLock<HashMap<TypeId, Arc<RequestFactoryFn>>>>,
    transient_factories: Arc<RwLock<HashMap<TypeId, Arc<TransientFactoryFn>>>>,

    /*
    We keep a set of registered type names mainly for debugging and error reporting.
    It helps us tell the user *which* provider is missing by name.
    */
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
    /// Creates a new, empty container.
    ///
    /// This is equivalent to `Container::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a "scoped" child container.
    ///
    /// A scoped container shares the underlying singleton registry (`inner`) with its parent
    /// but has its own empty `overrides` map.
    ///
    /// This is used during HTTP requests to create a context where request-scoped providers
    /// can be cached for the duration of that single request without affecting the global state.
    pub fn scoped(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            request_factories: Arc::clone(&self.request_factories),
            transient_factories: Arc::clone(&self.transient_factories),
            names: Arc::clone(&self.names),
        }
    }

    /// Registers a value (singleton) into the container.
    ///
    /// The value must be thread-safe (`Send + Sync`) and `'static`.
    ///
    /// # Example
    /// ```rust
    /// container.register(AppConfig::default())?;
    /// ```
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

    /// Replaces an existing registration with a new value.
    ///
    /// Unlike `register`, this will not error if the type is already present.
    /// It effectively updates the singleton instance.
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

    /// Overrides a value in the current scope.
    ///
    /// If called on a global container, it works like `replace` but stores the value
    /// in the `overrides` map, which takes precedence over `inner`.
    ///
    /// If called on a `scoped()` container, the override only exists for that scope.
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

    /// Checks if a type with the given name is registered.
    ///
    /// This relies on `std::any::type_name` matching what was stored.
    pub fn is_type_registered_name(&self, type_name: &'static str) -> Result<bool, ContainerError> {
        let names = self
            .names
            .read()
            .map_err(|_| ContainerError::ReadLockPoisoned)?;
        Ok(names.contains(type_name))
    }

    /// Registers a factory for request-scoped providers.
    ///
    /// A request-scoped provider is created once per `scoped()` container (i.e., once per request)
    /// and then cached within that scope.
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

    /// Registers a factory for transient providers.
    ///
    /// A transient provider is created anew every single time it is resolved.
    /// It is never cached.
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

    /// Resolves (retrieves) a registered provider.
    ///
    /// The search order is:
    /// 1. Overrides (scoped instances)
    /// 2. Singletons (global instances)
    /// 3. Request-scoped factories (create and cache in overrides if found)
    /// 4. Transient factories (create new instance)
    ///
    /// Returns an `Arc<T>` so the service can be cheaply shared.
    pub fn resolve<T>(&self) -> Result<Arc<T>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        /*
        Step 1: Check overrides.
        If we are in a request scope, this is where request-scoped instances live.
        */
        if let Some(value) = self.resolve_from_map::<T>(&self.overrides)? {
            return Ok(value);
        }

        /*
        Step 2: Check global singletons.
        This is the most common case for stateless services.
        */
        if let Some(value) = self.resolve_from_map::<T>(&self.inner)? {
            return Ok(value);
        }

        /*
        Step 3: Check request-scoped factories.
        If found, we run the factory, cache the result in `overrides`, and return it.
        */
        if let Some(value) = self.resolve_from_request_factory::<T>()? {
            return Ok(value);
        }

        /*
        Step 4: Check transient factories.
        If found, we run the factory and return a fresh instance.
        */
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

        let value = value
            .downcast::<T>()
            .map_err(|_| ContainerError::DowncastFailed {
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
        let typed = value
            .downcast::<T>()
            .map_err(|_| ContainerError::DowncastFailed {
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
        let typed = value
            .downcast::<T>()
            .map_err(|_| ContainerError::DowncastFailed {
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
