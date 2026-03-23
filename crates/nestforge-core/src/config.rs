use crate::{ConfigError, Container};
use anyhow::Result;
use nestforge_config::FromEnv;

pub trait Configurable: FromEnv + Send + Sync + 'static {
    fn load(options: &nestforge_config::ConfigOptions) -> Result<Self, ConfigError>;
}

impl<T: FromEnv + Send + Sync + 'static> Configurable for T {
    fn load(options: &nestforge_config::ConfigOptions) -> Result<Self, ConfigError> {
        let env = nestforge_config::EnvStore::load_with_options(options)?;
        let config = Self::from_env(&env)?;
        config.validate().map_err(|e| {
            let issues = e
                .field_errors()
                .iter()
                .map(|(key, errors)| nestforge_config::EnvValidationIssue {
                    key: key.to_string(),
                    message: errors
                        .first()
                        .and_then(|err| err.message.as_ref())
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Validation failed for {}", key)),
                })
                .collect();
            ConfigError::Validation { issues }
        })?;
        Ok(config)
    }
}

pub fn register_config<T: Configurable>(
    container: &Container,
    options: nestforge_config::ConfigOptions,
) -> Result<()> {
    let config = T::load(&options)?;
    container.register(config).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nestforge_config::ConfigOptions;
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Clone, Deserialize, Validate)]
    struct TestConfig {
        #[validate(length(min = 1, message = "TEST_VALUE cannot be empty"))]
        test_value: String,
        port: u16,
    }

    impl FromEnv for TestConfig {
        fn from_env(
            env: &nestforge_config::EnvStore,
        ) -> Result<Self, nestforge_config::ConfigError> {
            Ok(Self {
                test_value: env.get("TEST_VALUE").unwrap_or("default").to_string(),
                port: env.get("PORT").and_then(|v| v.parse().ok()).unwrap_or(8080),
            })
        }

        fn config_key() -> &'static str {
            "TestConfig"
        }
    }

    #[test]
    fn test_register_config_loads_from_env() {
        std::env::set_var("TEST_VALUE", "hello");
        std::env::set_var("PORT", "3000");

        let container = Container::new();
        let options = ConfigOptions::new();

        let result = register_config::<TestConfig>(&container, options);
        assert!(result.is_ok());

        let resolved = container.resolve::<TestConfig>();
        assert!(resolved.is_ok());
        assert_eq!(resolved.unwrap().test_value, "hello");

        std::env::remove_var("TEST_VALUE");
        std::env::remove_var("PORT");
    }
}
