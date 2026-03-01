use std::{marker::PhantomData, sync::Arc};

use anyhow::Result;
use nestforge_core::{initialize_module_graph, Container, ContainerError, ModuleDefinition};

type OverrideFn = Box<dyn Fn(&Container) -> Result<()> + Send + Sync>;

pub struct TestFactory<M: ModuleDefinition> {
    overrides: Vec<OverrideFn>,
    _marker: PhantomData<M>,
}

impl<M: ModuleDefinition> TestFactory<M> {
    pub fn create() -> Self {
        Self {
            overrides: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn override_provider<T>(mut self, value: T) -> Self
    where
        T: Send + Sync + Clone + 'static,
    {
        self.overrides.push(Box::new(move |container| {
            container.override_value(value.clone())?;
            Ok(())
        }));
        self
    }

    pub fn build(self) -> Result<TestingModule> {
        let container = Container::new();

        for override_fn in self.overrides {
            override_fn(&container)?;
        }

        let _ = initialize_module_graph::<M>(&container)?;
        Ok(TestingModule { container })
    }
}

#[derive(Clone)]
pub struct TestingModule {
    container: Container,
}

impl TestingModule {
    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn resolve<T>(&self) -> Result<Arc<T>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        self.container.resolve::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nestforge_core::{register_provider, Provider};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct AppConfig {
        app_name: &'static str,
    }

    struct AppModule;
    impl ModuleDefinition for AppModule {
        fn register(container: &Container) -> Result<()> {
            container.register(AppConfig {
                app_name: "default",
            })?;
            Ok(())
        }
    }

    #[test]
    fn builds_testing_module_and_resolves_default_provider() {
        let module = TestFactory::<AppModule>::create()
            .build()
            .expect("test module should build");

        let config = module
            .resolve::<AppConfig>()
            .expect("config should resolve");
        assert_eq!(
            *config,
            AppConfig {
                app_name: "default"
            }
        );
    }

    #[test]
    fn overrides_provider_value() {
        let module = TestFactory::<AppModule>::create()
            .override_provider(AppConfig { app_name: "test" })
            .build()
            .expect("test module should build with overrides");

        let config = module
            .resolve::<AppConfig>()
            .expect("config should resolve");
        assert_eq!(*config, AppConfig { app_name: "test" });
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct GreetingService {
        greeting: String,
    }

    struct FactoryModule;
    impl ModuleDefinition for FactoryModule {
        fn register(container: &Container) -> Result<()> {
            register_provider(
                container,
                Provider::factory(|container| {
                    let config = container.resolve::<AppConfig>()?;
                    Ok(GreetingService {
                        greeting: format!("hello {}", config.app_name),
                    })
                }),
            )?;
            Ok(())
        }
    }

    #[test]
    fn overrides_are_applied_before_factory_resolution() {
        let module = TestFactory::<FactoryModule>::create()
            .override_provider(AppConfig {
                app_name: "override",
            })
            .build()
            .expect("test module should build with transitive overrides");

        let greeting = module
            .resolve::<GreetingService>()
            .expect("greeting service should resolve");
        assert_eq!(greeting.greeting, "hello override");
    }
}
