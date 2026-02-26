use std::{collections::HashMap, env, fs, path::Path};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read env file `{path}`: {source}")]
    ReadEnvFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Missing required config key: {key}")]
    MissingKey { key: String },
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
            let content =
                fs::read_to_string(path_ref).map_err(|source| ConfigError::ReadEnvFile {
                    path: path_ref.display().to_string(),
                    source,
                })?;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = trimmed.split_once('=') {
                    values.entry(key.trim().to_string()).or_insert_with(|| {
                        value
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string()
                    });
                }
            }
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
}

pub fn load_config<T: FromEnv>() -> Result<T, ConfigError> {
    let env = EnvStore::load()?;
    T::from_env(&env)
}

pub struct ConfigModule;

impl ConfigModule {
    pub fn for_root<T: FromEnv>(options: ConfigOptions) -> Result<T, ConfigError> {
        let env = EnvStore::load_with_options(&options)?;
        T::from_env(&env)
    }

    pub fn env(options: ConfigOptions) -> Result<EnvStore, ConfigError> {
        EnvStore::load_with_options(&options)
    }
}
