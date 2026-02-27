# Config Module

NestForge config support is in `nestforge-config` and re-exported via `nestforge` with the `config` feature.

## Main Types

- `ConfigModule`
- `ConfigOptions`
- `EnvSchema`
- `FromEnv`

## Typical Usage

```rust
fn load_app_config() -> anyhow::Result<AppConfig> {
    let levels = vec!["trace", "debug", "info", "warn", "error"];
    let schema = nestforge::EnvSchema::new()
        .required("APP_NAME")
        .min_len("APP_NAME", 2)
        .one_of("LOG_LEVEL", &levels);

    Ok(nestforge::ConfigModule::for_root::<AppConfig>(
        nestforge::ConfigOptions::new().env_file(".env").validate_with(schema),
    )?)
}
```

## FromEnv

Your config struct implements `FromEnv`:

```rust
impl nestforge::FromEnv for AppConfig {
    fn from_env(env: &nestforge::EnvStore) -> Result<Self, nestforge::ConfigError> {
        Ok(Self {
            app_name: env.get("APP_NAME").unwrap_or("NestForge").to_string(),
            log_level: env.get("LOG_LEVEL").unwrap_or("info").to_string(),
        })
    }
}
```

## Validation Rules

`EnvSchema` supports:

- `.required("KEY")`
- `.min_len("KEY", min)`
- `.one_of("KEY", &[...])`

When validation fails, startup returns `ConfigError::Validation` with issue details.

## Env Sources

By default, options include process env + `.env` file values.

You can customize this with `ConfigOptions`.
