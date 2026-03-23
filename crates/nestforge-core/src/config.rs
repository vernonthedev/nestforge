use crate::Container;
use anyhow::Result;
use nestforge_config::{ConfigError, ConfigOptions, EnvStore, FromEnv};

pub trait Configurable: FromEnv + Send + Sync + 'static {
    fn load(options: &ConfigOptions) -> Result<Self, ConfigError>;
}

impl<T: FromEnv + Send + Sync + 'static> Configurable for T {
    fn load(options: &ConfigOptions) -> Result<Self, ConfigError> {
        let env = EnvStore::load_with_options(options)?;
        T::from_env(&env)
    }
}

pub fn register_config<T: Configurable>(
    container: &Container,
    options: ConfigOptions,
) -> Result<()> {
    let config = T::load(&options)?;
    container.register(config).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nestforge_config::ConfigOptions;

    #[derive(Debug, Clone)]
    struct TestConfig {
        test_value: String,
    }

    impl FromEnv for TestConfig {
        fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
            Ok(Self {
                test_value: env.get("TEST_VALUE").unwrap_or("default").to_string(),
            })
        }

        fn config_key() -> &'static str {
            "TestConfig"
        }
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                test_value: "default".to_string(),
            }
        }
    }

    #[test]
    fn test_register_config_loads_from_env() {
        std::env::set_var("TEST_VALUE", "hello");

        let container = Container::new();
        let options = ConfigOptions::new();

        let result = register_config::<TestConfig>(&container, options);
        assert!(result.is_ok());

        let resolved = container.resolve::<TestConfig>();
        assert!(resolved.is_ok());
        assert_eq!(resolved.unwrap().test_value, "hello");

        std::env::remove_var("TEST_VALUE");
    }
}
