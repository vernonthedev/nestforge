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

#[derive(Clone, Debug, Default)]
pub struct EnvStore {
    values: HashMap<String, String>,
}

impl EnvStore {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_file(".env")
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let mut values = env::vars().collect::<HashMap<_, _>>();

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
