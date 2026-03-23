use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read env file `{path}`: {source}")]
    ReadEnvFile {
        path: String,
        #[source]
        source: dotenvy::Error,
    },
    #[error("Missing config key: {key}")]
    MissingKey { key: String },
}

#[derive(Clone, Debug, Default)]
pub struct ConfigService {
    values: Arc<HashMap<String, String>>,
}

impl ConfigService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with_options(&ConfigOptions::default())
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
                        values.insert(key, value);
                    }
                });
        }

        Ok(Self {
            values: Arc::new(values),
        })
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    pub fn get_or_panic(&self, key: &str) -> String {
        self.get(key)
            .unwrap_or_else(|| panic!("Missing required config key: {key}"))
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn get_as<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    pub fn get_as_or<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        self.get_as(key).unwrap_or(default)
    }
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

pub struct ConfigModule;

impl ConfigModule {
    pub fn for_root() -> ConfigOptions {
        ConfigOptions::new()
    }

    pub fn for_feature() -> ConfigOptions {
        ConfigOptions::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_service_load() {
        std::env::set_var("APP_NAME", "TestApp");
        std::env::set_var("APP_PORT", "8080");

        let config = ConfigService::load().unwrap();

        assert_eq!(config.get("APP_NAME"), Some("TestApp".to_string()));
        assert_eq!(config.get_as::<u16>("APP_PORT"), Some(8080));

        std::env::remove_var("APP_NAME");
        std::env::remove_var("APP_PORT");
    }

    #[test]
    fn test_config_service_defaults() {
        let config = ConfigService::new();

        assert_eq!(config.get("MISSING"), None);
        assert_eq!(config.get_or("MISSING", "default"), "default");
        assert_eq!(config.get_as_or("MISSING", 3000u16), 3000);
    }

    #[test]
    fn test_config_options_builder() {
        let options = ConfigOptions::new().env_file(".env.test");
        assert_eq!(options.env_file_path, ".env.test");
    }
}
