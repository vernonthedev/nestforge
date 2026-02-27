use std::marker::PhantomData;

use anyhow::{anyhow, Result};

use crate::{framework_log, Container};

pub struct Provider;

pub struct ValueProvider<T> {
    value: T,
}

pub struct FactoryProvider<T, F> {
    factory: F,
    _marker: PhantomData<fn() -> T>,
}

impl Provider {
    pub fn value<T>(value: T) -> ValueProvider<T>
    where
        T: Send + Sync + 'static,
    {
        ValueProvider { value }
    }

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
}

pub trait RegisterProvider {
    fn register(self, container: &Container) -> Result<()>;
}

impl<T> RegisterProvider for ValueProvider<T>
where
    T: Send + Sync + 'static,
{
    fn register(self, container: &Container) -> Result<()> {
        framework_log(format!(
            "Registering service {}.",
            std::any::type_name::<T>()
        ));
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
        framework_log(format!(
            "Registering service {} (factory).",
            std::any::type_name::<T>()
        ));
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
}
