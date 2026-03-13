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

pub trait Injectable: Send + Sync + 'static {
    fn register(container: &Container) -> Result<()>;

    fn export_name() -> &'static str {
        std::any::type_name::<Self>()
    }
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
            let value: FactoryService = Result::<FactoryService>::Ok(FactoryService("ready"))
                .into_injectable_result()?;
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
