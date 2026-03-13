use anyhow::Result;

use crate::{framework_log_event, Container};

pub trait IntoInjectableResult<T> {
    fn into_injectable_result(self) -> Result<T>;
}

impl<T> IntoInjectableResult<T> for T {
    fn into_injectable_result(self) -> Result<T> {
        Ok(self)
    }
}

impl<T> IntoInjectableResult<T> for Result<T> {
    fn into_injectable_result(self) -> Result<T> {
        self
    }
}

/**
 * `Injectable` is the core trait of NestForge's dependency injection system.
 *
 * The `#[injectable]` macro automatically implements this trait for the decorated struct.
 * It tells NestForge how to create an instance of a service and register it within
 * the global `Container`.
 *
 * Manual implementation is possible for complex registration logic that the macro
 * does not yet support.
 *
 * ### Manual Implementation Example
 * ```rust
 * use nestforge::{Injectable, Container};
 *
 * struct MyService;
 *
 * impl Injectable for MyService {
 *     fn register(container: &Container) -> anyhow::Result<()> {
 *         // Registration logic goes here
 *         container.register(MyService)?;
 *         Ok(())
 *     }
 * }
 * ```
 */
pub trait Injectable: Send + Sync + 'static {
    /// Registers the provider into the provided `Container`.
    /// This is usually called during the module initialization phase.
    fn register(container: &Container) -> anyhow::Result<()>;
}

pub fn register_injectable<T>(container: &Container) -> Result<()>
where
    T: Injectable,
{
    framework_log_event(
        "injectable_register",
        &[("type", std::any::type_name::<T>().to_string())],
    );
    T::register(container)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct DefaultService;

    impl Injectable for DefaultService {
        fn register(container: &Container) -> Result<()> {
            container.register(Self)?;
            Ok(())
        }
    }

    struct FactoryService(&'static str);

    impl Injectable for FactoryService {
        fn register(container: &Container) -> Result<()> {
            let value: FactoryService =
                Result::<FactoryService>::Ok(FactoryService("ready")).into_injectable_result()?;
            container.register(value)?;
            Ok(())
        }
    }

    #[test]
    fn register_injectable_stores_service_in_container() {
        let container = Container::new();

        register_injectable::<DefaultService>(&container).expect("injectable should register");

        assert!(container.resolve::<DefaultService>().is_ok());
    }

    #[test]
    fn into_injectable_result_accepts_plain_values() {
        let value = FactoryService("plain")
            .into_injectable_result()
            .expect("plain value should convert");

        assert_eq!(value.0, "plain");
    }

    #[test]
    fn into_injectable_result_accepts_anyhow_results() {
        let container = Container::new();

        register_injectable::<FactoryService>(&container).expect("factory should register");

        let service = container
            .resolve::<FactoryService>()
            .expect("factory service should resolve");
        assert_eq!(service.0, "ready");
    }
}
