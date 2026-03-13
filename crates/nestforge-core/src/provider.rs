use std::marker::PhantomData;

use anyhow::{anyhow, Result};

use crate::{framework_log_event, Container};

/**
 * Provider Helper Struct
 *
 * A helper struct for creating different types of providers.
 * Use the static methods `Provider::value()`, `Provider::factory()`,
 * `Provider::request_factory()`, and `Provider::transient_factory()`
 * when defining module providers or registering them manually.
 *
 * # Example
 * ```rust
 * impl ModuleDefinition for AppModule {
 *     fn register(container: &Container) -> Result<()> {
 *         register_provider(container, Provider::value(AppConfig::default()))?;
 *         register_provider(container, Provider::factory(|c| Ok(MyService::new(c))))?;
 *         Ok(())
 *     }
 * }
 * ```
 */
pub struct Provider;

/**
 * ValueProvider
 *
 * A provider that registers an existing value as a singleton.
 * The value is registered directly into the container and shared
 * across all resolutions.
 */
pub struct ValueProvider<T> {
    value: T,
}

/**
 * FactoryProvider
 *
 * A provider that uses a factory function to create a singleton.
 * The factory runs once, immediately upon registration, and the
 * resulting instance is stored as a singleton.
 */
pub struct FactoryProvider<T, F> {
    factory: F,
    _marker: PhantomData<fn() -> T>,
}

/**
 * RequestFactoryProvider
 *
 * A provider that creates a new instance for every HTTP request.
 * The instance is cached for the duration of that request and shared
 * within that request's scope.
 */
pub struct RequestFactoryProvider<T, F> {
    factory: F,
    _marker: PhantomData<fn() -> T>,
}

/**
 * TransientFactoryProvider
 *
 * A provider that creates a new instance every time it is resolved.
 * Unlike singletons or request-scoped providers, transient instances
 * are never cached - a fresh instance is created on each resolution.
 */
pub struct TransientFactoryProvider<T, F> {
    factory: F,
    _marker: PhantomData<fn() -> T>,
}

impl Provider {
    /**
     * Creates a value provider from an existing value.
     *
     * The value will be registered as a singleton in the container.
     *
     * # Type Parameters
     * - `T`: The type to register (must be Send + Sync + 'static)
     *
     * # Arguments
     * - `value`: The value to register as a singleton
     */
    pub fn value<T>(value: T) -> ValueProvider<T>
    where
        T: Send + Sync + 'static,
    {
        ValueProvider { value }
    }

    /**
     * Creates a factory provider.
     *
     * The factory receives the Container and returns Result<T>.
     * It is executed immediately when the module registers its providers.
     * The result is stored as a singleton.
     *
     * # Type Parameters
     * - `T`: The type to create (must be Send + Sync + 'static)
     * - `F`: The factory function type
     */
    pub fn factory<T, F>(factory: F) -> FactoryProvider<T, F>
    where
        T: Send + Sync + 'static,
        F: FnOnce(&Container) -> Result<T> + Send + 'static,
    {
        FactoryProvider {
            factory,
            _marker: PhantomData,
        }
    }

    /**
     * Creates a request-scoped provider.
     *
     * The factory is executed once per request (per scoped container).
     * The created instance is cached for the duration of that request.
     *
     * # Type Parameters
     * - `T`: The type to create (must be Send + Sync + 'static)
     * - `F`: The factory function type
     */
    pub fn request_factory<T, F>(factory: F) -> RequestFactoryProvider<T, F>
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        RequestFactoryProvider {
            factory,
            _marker: PhantomData,
        }
    }

    /**
     * Creates a transient provider.
     *
     * The factory is executed every time the type is resolved via
     * container.resolve(). A new instance is created each time.
     *
     * # Type Parameters
     * - `T`: The type to create (must be Send + Sync + 'static)
     * - `F`: The factory function type
     */
    pub fn transient_factory<T, F>(factory: F) -> TransientFactoryProvider<T, F>
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        TransientFactoryProvider {
            factory,
            _marker: PhantomData,
        }
    }
}

/**
 * RegisterProvider Trait
 *
 * A trait for types that can register themselves into a Container.
 * This is implemented by all provider types returned by Provider:: methods.
 */
pub trait RegisterProvider {
    /**
     * Registers this provider into the given container.
     */
    fn register(self, container: &Container) -> Result<()>;
}

impl<T> RegisterProvider for ValueProvider<T>
where
    T: Send + Sync + 'static,
{
    fn register(self, container: &Container) -> Result<()> {
        framework_log_event(
            "provider_register",
            &[("type", std::any::type_name::<T>().to_string())],
        );
        container.register(self.value)?;
        Ok(())
    }
}

impl<T, F> RegisterProvider for FactoryProvider<T, F>
where
    T: Send + Sync + 'static,
    F: FnOnce(&Container) -> Result<T> + Send + 'static,
{
    fn register(self, container: &Container) -> Result<()> {
        framework_log_event(
            "provider_register_factory",
            &[("type", std::any::type_name::<T>().to_string())],
        );
        let value = (self.factory)(container).map_err(|err| {
            anyhow!(
                "Failed to build provider `{}`: {}",
                std::any::type_name::<T>(),
                err
            )
        })?;
        container.register(value)?;
        Ok(())
    }
}

impl<T, F> RegisterProvider for RequestFactoryProvider<T, F>
where
    T: Send + Sync + 'static,
    F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
{
    fn register(self, container: &Container) -> Result<()> {
        framework_log_event(
            "provider_register_request_factory",
            &[("type", std::any::type_name::<T>().to_string())],
        );
        container
            .register_request_factory::<T, _>(move |container| {
                (self.factory)(container).map_err(|err| {
                    anyhow!(
                        "Failed to build request-scoped provider `{}`: {}",
                        std::any::type_name::<T>(),
                        err
                    )
                })
            })
            .map_err(|err| anyhow!("Failed to register request-scoped provider: {err}"))?;
        Ok(())
    }
}

impl<T, F> RegisterProvider for TransientFactoryProvider<T, F>
where
    T: Send + Sync + 'static,
    F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
{
    fn register(self, container: &Container) -> Result<()> {
        framework_log_event(
            "provider_register_transient_factory",
            &[("type", std::any::type_name::<T>().to_string())],
        );
        container
            .register_transient_factory::<T, _>(move |container| {
                (self.factory)(container).map_err(|err| {
                    anyhow!(
                        "Failed to build transient provider `{}`: {}",
                        std::any::type_name::<T>(),
                        err
                    )
                })
            })
            .map_err(|err| anyhow!("Failed to register transient provider: {err}"))?;
        Ok(())
    }
}

/// Helper function to register a provider into a container.
pub fn register_provider<P>(container: &Container, provider: P) -> Result<()>
where
    P: RegisterProvider,
{
    provider.register(container)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct AppConfig {
        app_name: &'static str,
    }

    struct AppService {
        config_name: &'static str,
    }

    #[test]
    fn registers_value_provider() {
        let container = Container::new();
        let result = register_provider(
            &container,
            Provider::value(AppConfig {
                app_name: "nestforge",
            }),
        );

        assert!(result.is_ok(), "value provider registration should succeed");
        let config = container
            .resolve::<AppConfig>()
            .expect("config should be registered");
        assert_eq!(config.app_name, "nestforge");
    }

    #[test]
    fn registers_factory_provider() {
        let container = Container::new();
        register_provider(
            &container,
            Provider::value(AppConfig {
                app_name: "nestforge",
            }),
        )
        .expect("seed config");

        let result = register_provider(
            &container,
            Provider::factory(|c| {
                let cfg = c.resolve::<AppConfig>()?;
                Ok(AppService {
                    config_name: cfg.app_name,
                })
            }),
        );

        assert!(
            result.is_ok(),
            "factory provider registration should succeed"
        );
        let service = container
            .resolve::<AppService>()
            .expect("service should be registered");
        assert_eq!(service.config_name, "nestforge");
    }

    #[test]
    fn factory_error_includes_type_name() {
        let container = Container::new();
        let err = register_provider(
            &container,
            Provider::factory::<AppService, _>(|_| Err(anyhow!("boom"))),
        )
        .expect_err("factory should fail");

        assert!(err.to_string().contains("AppService"));
    }

    #[test]
    fn registers_request_factory_provider() {
        #[derive(Clone)]
        struct RequestId(&'static str);

        struct RequestService(&'static str);

        let container = Container::new();
        register_provider(
            &container,
            Provider::request_factory(|c| {
                let request_id = c.resolve::<RequestId>()?;
                Ok(RequestService(request_id.0))
            }),
        )
        .expect("request factory should register");

        let scoped = container.scoped();
        scoped
            .override_value(RequestId("req-42"))
            .expect("request id should be set");

        let service = scoped
            .resolve::<RequestService>()
            .expect("request service should resolve");
        assert_eq!(service.0, "req-42");
    }

    #[test]
    fn registers_transient_factory_provider() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        struct TransientService(usize);

        let container = Container::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_for_factory = Arc::clone(&counter);

        register_provider(
            &container,
            Provider::transient_factory(move |_| {
                let value = counter_for_factory.fetch_add(1, Ordering::Relaxed) + 1;
                Ok(TransientService(value))
            }),
        )
        .expect("transient factory should register");

        let first = container
            .resolve::<TransientService>()
            .expect("first transient should resolve");
        let second = container
            .resolve::<TransientService>()
            .expect("second transient should resolve");

        assert_eq!(first.0, 1);
        assert_eq!(second.0, 2);
    }
}
