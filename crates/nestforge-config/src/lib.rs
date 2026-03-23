use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read env file `{path}`: {source}")]
    ReadEnvFile {
        path: String,
        #[source]
        source: dotenvy::Error,
    },
    #[error("Missing required config key: {key}")]
    MissingKey { key: String },
    #[error("Environment validation failed")]
    Validation { issues: Vec<EnvValidationIssue> },
    #[error("Serde deserialization failed: {0}")]
    Deserialization(String),
    #[error("Failed to load configuration: {0}")]
    LoadFailed(String),
}

#[derive(Clone, Debug)]
pub struct EnvValidationIssue {
    pub key: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ConfigOptions {
    pub env_file_path: String,
    pub include_process_env: bool,
    pub prefix: Option<String>,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            env_file_path: ".env".to_string(),
            include_process_env: true,
            prefix: None,
        }
    }
}

impl ConfigOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env_file(mut self, path: impl Into<String>) -> Self {
        self.env_file_path = path.into();
        self
    }

    pub fn without_process_env(mut self) -> Self {
        self.include_process_env = false;
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnvStore {
    values: HashMap<String, String>,
}

impl EnvStore {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with_options(&ConfigOptions::default())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        Self::load_with_options(&ConfigOptions::new().env_file(path.as_ref().display().to_string()))
    }

    pub fn load_with_options(options: &ConfigOptions) -> Result<Self, ConfigError> {
        let path_ref = Path::new(&options.env_file_path);
        let mut values = if options.include_process_env {
            env::vars().collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        if path_ref.exists() {
            dotenvy::from_path_iter(path_ref)
                .map_err(|source| ConfigError::ReadEnvFile {
                    path: path_ref.display().to_string(),
                    source,
                })?
                .for_each(|result| {
                    if let Ok((key, value)) = result {
                        values.entry(key).or_insert(value);
                    }
                });
        }

        Ok(Self { values })
    }

    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            values: pairs.into_iter().collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn require(&self, key: &str) -> Result<&str, ConfigError> {
        self.get(key).ok_or_else(|| ConfigError::MissingKey {
            key: key.to_string(),
        })
    }

    pub fn get_with_prefix(&self, prefix: &str, key: &str) -> Option<&str> {
        let full_key = format!("{}_{}", prefix, key);
        self.get(&full_key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

pub trait FromEnv: Sized + Validate {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError>;
    fn config_key() -> &'static str;
}

pub trait ConfigField: Sized {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError>;
    fn from_env_required(env: &EnvStore, key: &str) -> Result<Self, ConfigError>;
    fn from_env_or(env: &EnvStore, key: &str, default: Self) -> Result<Self, ConfigError>;
    fn validate_value(_value: &Self) -> Result<(), ConfigError> {
        Ok(())
    }
}

macro_rules! impl_config_field_for_primitives {
    ($($ty:ty),*) => {
        $(
            impl ConfigField for $ty {
                fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
                    let key = std::any::type_name::<Self>().split("::").last().unwrap_or("UNKNOWN");
                    Self::from_env_required(env, key)
                }

                fn from_env_required(env: &EnvStore, key: &str) -> Result<Self, ConfigError> {
                    let value = env.require(key)?;
                    value.parse().map_err(|_| ConfigError::Deserialization(
                        format!("Failed to parse '{}' as {}", value, std::any::type_name::<Self>())
                    ))
                }

                fn from_env_or(env: &EnvStore, key: &str, default: Self) -> Result<Self, ConfigError> {
                    match env.get(key) {
                        Some(value) => value.parse().map_err(|_| ConfigError::Deserialization(
                            format!("Failed to parse '{}' as {}", value, std::any::type_name::<Self>())
                        )),
                        None => Ok(default),
                    }
                }
            }
        )*
    };
}

impl_config_field_for_primitives!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

impl ConfigField for String {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
        let key = std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UNKNOWN");
        Self::from_env_required(env, key)
    }

    fn from_env_required(env: &EnvStore, key: &str) -> Result<Self, ConfigError> {
        let value = env.require(key)?;
        Ok(value.to_string())
    }

    fn from_env_or(env: &EnvStore, key: &str, default: Self) -> Result<Self, ConfigError> {
        match env.get(key) {
            Some(value) => Ok(value.to_string()),
            None => Ok(default),
        }
    }
}

impl ConfigField for bool {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
        let key = std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UNKNOWN");
        Self::from_env_required(env, key)
    }

    fn from_env_required(env: &EnvStore, key: &str) -> Result<Self, ConfigError> {
        let value = env.require(key)?;
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(ConfigError::Deserialization(format!(
                "Failed to parse '{}' as bool: expected true/false/1/0/yes/no/on/off",
                value
            ))),
        }
    }

    fn from_env_or(env: &EnvStore, key: &str, default: Self) -> Result<Self, ConfigError> {
        match env.get(key) {
            Some(value) => match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Ok(true),
                "false" | "0" | "no" | "off" => Ok(false),
                _ => Err(ConfigError::Deserialization(format!(
                    "Failed to parse '{}' as bool: expected true/false/1/0/yes/no/on/off",
                    value
                ))),
            },
            None => Ok(default),
        }
    }
}

impl<T: ConfigField> ConfigField for Option<T> {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
        let key = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("UNKNOWN");
        Self::from_env_required(env, key)
    }

    fn from_env_required(env: &EnvStore, key: &str) -> Result<Self, ConfigError> {
        match env.get(key) {
            Some(_value) => Ok(Some(T::from_env_required(env, key)?)),
            None => Ok(None),
        }
    }

    fn from_env_or(_env: &EnvStore, _key: &str, default: Self) -> Result<Self, ConfigError> {
        Ok(default)
    }
}

pub fn load_config<T: FromEnv>() -> Result<T, ConfigError> {
    let env_store = EnvStore::load()?;
    T::from_env(&env_store)
}

pub struct ConfigModule;

impl ConfigModule {
    pub fn for_root<T: FromEnv>(options: ConfigOptions) -> Result<T, ConfigError> {
        let env = EnvStore::load_with_options(&options)?;
        let config = T::from_env(&env)?;
        config.validate().map_err(|e| {
            let issues = e
                .field_errors()
                .iter()
                .map(|(key, errors)| EnvValidationIssue {
                    key: key.to_string(),
                    message: errors
                        .first()
                        .and_then(|e| e.message.as_ref())
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Validation failed for {}", key)),
                })
                .collect();
            ConfigError::Validation { issues }
        })?;
        Ok(config)
    }

    pub fn for_feature<T: FromEnv>(options: ConfigOptions) -> Result<T, ConfigError> {
        Self::for_root(options)
    }

    pub fn env(options: ConfigOptions) -> Result<EnvStore, ConfigError> {
        EnvStore::load_with_options(&options)
    }
}

pub struct ConfigEntry {
    pub type_name: &'static str,
    pub loader: fn(&EnvStore) -> Result<Box<dyn ConfigValue>, ConfigError>,
}

pub trait ConfigDescriptor: Send + Sync + 'static {
    fn load_config(env: &EnvStore) -> Result<Box<dyn ConfigValue>, ConfigError>;
    fn type_name() -> &'static str;
}

pub trait ConfigValue: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Send + Sync + 'static> ConfigValue for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ConfigRegistry;

impl ConfigRegistry {
    pub fn load<T: FromEnv + 'static>(env: &EnvStore) -> Result<T, ConfigError> {
        let config = T::from_env(env)?;
        config.validate().map_err(|e| {
            let issues = e
                .field_errors()
                .iter()
                .map(|(key, errors)| EnvValidationIssue {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize, Validate)]
    pub struct TestConfig {
        #[validate(length(min = 1, message = "DATABASE_URL cannot be empty"))]
        pub database_url: String,
        #[serde(default = "default_port")]
        pub port: u16,
    }

    fn default_port() -> u16 {
        5432
    }

    impl FromEnv for TestConfig {
        fn from_env(env: &EnvStore) -> Result<Self, ConfigError> {
            Ok(Self {
                database_url: env
                    .get("DATABASE_URL")
                    .unwrap_or("postgres://localhost/db")
                    .to_string(),
                port: env.get("PORT").and_then(|v| v.parse().ok()).unwrap_or(5432),
            })
        }

        fn config_key() -> &'static str {
            "TestConfig"
        }
    }

    #[test]
    fn test_env_store_from_pairs() {
        let store = EnvStore::from_pairs(vec![
            (
                "DATABASE_URL".to_string(),
                "postgres://localhost/db".to_string(),
            ),
            ("PORT".to_string(), "8080".to_string()),
        ]);

        assert_eq!(store.get("DATABASE_URL"), Some("postgres://localhost/db"));
        assert_eq!(store.get("PORT"), Some("8080"));
        assert_eq!(store.get("MISSING"), None);
    }

    #[test]
    fn test_config_options_builder() {
        let options = ConfigOptions::new()
            .env_file(".env.test")
            .with_prefix("APP");

        assert_eq!(options.env_file_path, ".env.test");
        assert_eq!(options.prefix, Some("APP".to_string()));
    }

    #[test]
    fn test_test_config_from_env() {
        let store = EnvStore::from_pairs(vec![
            (
                "DATABASE_URL".to_string(),
                "postgres://localhost/mydb".to_string(),
            ),
            ("PORT".to_string(), "5433".to_string()),
        ]);
        let config = TestConfig::from_env(&store).unwrap();

        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.port, 5433);
    }

    #[test]
    fn test_env_store_with_prefix() {
        let store = EnvStore::from_pairs(vec![
            ("APP_HOST".to_string(), "0.0.0.0".to_string()),
            ("APP_PORT".to_string(), "8080".to_string()),
        ]);

        assert_eq!(store.get_with_prefix("APP", "HOST"), Some("0.0.0.0"));
        assert_eq!(store.get_with_prefix("APP", "PORT"), Some("8080"));
        assert_eq!(store.get_with_prefix("DB", "HOST"), None);
    }

    #[test]
    fn test_test_config_defaults() {
        let store = EnvStore::from_pairs(vec![]);
        let config = TestConfig::from_env(&store).unwrap();

        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.port, 5432);
    }
}
