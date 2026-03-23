use std::collections::HashMap;
use std::env;
use std::path::Path;
use thiserror::Error;

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
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            env_file_path: ".env".to_string(),
            include_process_env: true,
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
}

pub trait FromEnv: Sized {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError>;
    fn config_key() -> &'static str;
}

pub fn load<T: FromEnv>() -> Result<T, ConfigError> {
    let env_store = EnvStore::load()?;
    T::from_env(&env_store)
}

pub struct ConfigModule;

impl ConfigModule {
    pub fn for_root() -> ConfigOptions {
        ConfigOptions::new()
    }

    pub fn for_feature() -> ConfigOptions {
        ConfigOptions::new()
    }

    pub fn env(options: ConfigOptions) -> Result<EnvStore, ConfigError> {
        EnvStore::load_with_options(&options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct TestConfig {
        pub database_url: String,
        pub port: u16,
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
        let options = ConfigOptions::new().env_file(".env.test");

        assert_eq!(options.env_file_path, ".env.test");
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
    fn test_test_config_defaults() {
        let store = EnvStore::from_pairs(vec![]);
        let config = TestConfig::from_env(&store).unwrap();

        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.port, 5432);
    }
}
