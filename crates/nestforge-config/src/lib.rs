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
    #[error("Failed to parse env file `{path}` at line {line}: {source}")]
    ParseEnvFile {
        path: String,
        line: usize,
        #[source]
        source: dotenvy::Error,
    },
    #[error("Missing config key: {key}")]
    MissingKey { key: String },
    #[error("Failed to parse config key `{key}`: {value}")]
    ParseError { key: String, value: String },
}

#[derive(Clone, Debug, Default)]
pub struct EnvSchema {
    requirements: Vec<String>,
}

impl EnvSchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn required(&mut self, key: &str) -> &mut Self {
        self.requirements.push(key.to_string());
        self
    }
}

#[derive(Clone, Debug)]
pub struct EnvStore {
    values: HashMap<String, String>,
}

impl EnvStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

impl Default for EnvStore {
    fn default() -> Self {
        Self {
            values: env::vars().collect(),
        }
    }
}

impl From<ConfigService> for EnvStore {
    fn from(config: ConfigService) -> Self {
        Self {
            values: config.values,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvValidationIssue {
    pub key: String,
    pub message: String,
}

pub trait FromEnv: Sized {
    fn from_env(env: &EnvStore) -> Result<Self, ConfigError>;
}

#[derive(Clone, Debug, Default)]
pub struct ConfigService {
    values: HashMap<String, String>,
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
        let mut values: HashMap<String, String> = if options.include_process_env {
            env::vars().collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        if path_ref.exists() {
            let iter =
                dotenvy::from_path_iter(path_ref).map_err(|source| ConfigError::ReadEnvFile {
                    path: path_ref.display().to_string(),
                    source,
                })?;

            for result in iter {
                let (key, value) = result.map_err(|source| ConfigError::ParseEnvFile {
                    path: path_ref.display().to_string(),
                    line: 0,
                    source,
                })?;
                values.entry(key).or_insert(value);
            }
        }

        Ok(Self { values })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn get_string(&self, key: &str) -> String {
        self.get(key).map(|v| v.to_string()).unwrap_or_default()
    }

    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|v| v.to_string())
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_i32(&self, key: &str) -> i32 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_i32_or(&self, key: &str, default: i32) -> i32 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_u16(&self, key: &str) -> u16 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_u16_or(&self, key: &str, default: u16) -> u16 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_u32(&self, key: &str) -> u32 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_u32_or(&self, key: &str, default: u32) -> u32 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key)
            .map(|v| v == "true" || v == "1" || v == "yes")
            .unwrap_or(false)
    }

    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key)
            .map(|v| v == "true" || v == "1" || v == "yes")
            .unwrap_or(default)
    }

    pub fn get_usize(&self, key: &str) -> usize {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_usize_or(&self, key: &str, default: usize) -> usize {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_f64(&self, key: &str) -> f64 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0.0)
    }

    pub fn get_f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_isize(&self, key: &str) -> isize {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_isize_or(&self, key: &str, default: isize) -> isize {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_i64(&self, key: &str) -> i64 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_u64(&self, key: &str) -> u64 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    pub fn get_u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
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

    pub fn for_root_with_options(options: ConfigOptions) -> ConfigService {
        ConfigService::load_with_options(&options).expect("Failed to load configuration")
    }

    pub fn try_for_root_with_options(options: ConfigOptions) -> Result<ConfigService, ConfigError> {
        ConfigService::load_with_options(&options)
    }

    pub fn for_feature() -> ConfigOptions {
        ConfigOptions::new()
    }
}

pub fn load_config() -> ConfigService {
    ConfigModule::for_root_with_options(ConfigModule::for_root())
}

use std::sync::Arc;

pub struct Config<T> {
    _phantom: std::marker::PhantomData<T>,
    service: Arc<ConfigService>,
}

impl<T> Config<T> {
    pub fn new(service: ConfigService) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            service: Arc::new(service),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.service.get(key)
    }

    pub fn get_string(&self, key: &str) -> String {
        self.service.get_string(key)
    }

    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.service.get_string_or(key, default)
    }

    pub fn get_i32(&self, key: &str) -> i32 {
        self.service.get_i32(key)
    }

    pub fn get_i32_or(&self, key: &str, default: i32) -> i32 {
        self.service.get_i32_or(key, default)
    }

    pub fn get_u16(&self, key: &str) -> u16 {
        self.service.get_u16(key)
    }

    pub fn get_u16_or(&self, key: &str, default: u16) -> u16 {
        self.service.get_u16_or(key, default)
    }

    pub fn get_u32(&self, key: &str) -> u32 {
        self.service.get_u32(key)
    }

    pub fn get_u32_or(&self, key: &str, default: u32) -> u32 {
        self.service.get_u32_or(key, default)
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.service.get_bool(key)
    }

    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.service.get_bool_or(key, default)
    }

    pub fn get_usize(&self, key: &str) -> usize {
        self.service.get_usize(key)
    }

    pub fn get_usize_or(&self, key: &str, default: usize) -> usize {
        self.service.get_usize_or(key, default)
    }

    pub fn has(&self, key: &str) -> bool {
        self.service.has(key)
    }
}

impl<T> std::ops::Deref for Config<T> {
    type Target = ConfigService;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

pub fn register_config<T: Send + Sync + 'static>(
    name: &'static str,
    factory: fn() -> T,
) -> ConfigRegistration<T> {
    ConfigRegistration {
        name,
        _phantom: std::marker::PhantomData,
        factory,
    }
}

pub struct ConfigRegistration<T: Send + Sync + 'static> {
    #[allow(dead_code)]
    name: &'static str,
    _phantom: std::marker::PhantomData<T>,
    factory: fn() -> T,
}

impl<T: Send + Sync + 'static> ConfigRegistration<T> {
    pub fn load(&self) -> T {
        (self.factory)()
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

        assert_eq!(config.get("APP_NAME"), Some("TestApp"));
        assert_eq!(config.get_string("APP_NAME"), "TestApp");
        assert_eq!(config.get_u16("APP_PORT"), 8080);
        assert_eq!(config.get_u16_or("MISSING", 3000), 3000);
        assert!(config.has("APP_NAME"));
        assert!(!config.has("MISSING"));

        std::env::remove_var("APP_NAME");
        std::env::remove_var("APP_PORT");
    }

    #[test]
    fn test_config_service_defaults() {
        let config = ConfigService::new();

        assert_eq!(config.get_string("MISSING"), "");
        assert_eq!(config.get_string_or("MISSING", "default"), "default");
        assert_eq!(config.get_u16_or("MISSING", 3000), 3000);
        assert_eq!(config.get_bool_or("MISSING", true), true);
    }

    #[test]
    fn test_config_options_builder() {
        let options = ConfigOptions::new().env_file(".env.test");
        assert_eq!(options.env_file_path, ".env.test");
    }

    #[test]
    fn test_register_config() {
        let db_config = register_config("database", || DbConfig {
            host: "localhost".to_string(),
            port: 5432,
        });

        let config = db_config.load();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
    }

    #[derive(Debug, Clone)]
    struct DbConfig {
        host: String,
        port: u16,
    }
}
