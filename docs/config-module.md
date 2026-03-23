# Configuration Module

NestForge provides a NestJS-inspired configuration system via the `nestforge-config` crate.

## Installation

The config module is included by default in the `nestforge` crate.

## Quick Start

The easiest way to get started is to use the CLI to scaffold your project:

```bash
nestforge new my-app
```

This generates an `app_config.rs` file for you.

## Manual Setup

### 1. Create app_config.rs

```rust
// src/app_config.rs
use nestforge_config::{ConfigService, ConfigModule};

pub type AppConfig = ConfigService;

pub fn load_config() -> AppConfig {
    ConfigModule::for_root_with_options(ConfigModule::for_root().env_file(".env"))
}
```

### 2. Register in AppModule

```rust
// src/app_module.rs
use nestforge::module;
use crate::app_config::{AppConfig, load_config};

#[module(
    providers = [
        load_config() => AppConfig
    ],
    exports = [AppConfig]
)]
pub struct AppModule;
```

## Usage

### In Services

Inject `Config<AppConfig>` and use the intuitive getter methods:

```rust
use nestforge::prelude::*;

pub struct AppService {
    pub config: Config<AppConfig>,
}

impl AppService {
    pub fn get_app_name(&self) -> String {
        self.config.get_string_or("APP_NAME", "My App")
    }

    pub fn get_port(&self) -> u16 {
        self.config.get_u16_or("PORT", 3000)
    }

    pub fn is_debug(&self) -> bool {
        self.config.get_bool_or("DEBUG", false)
    }
}
```

## Getter Methods

The ConfigService provides type-safe getters with optional defaults:

| Method | Description | Default if missing |
|--------|-------------|-------------------|
| `get_string("KEY")` | Get string value | Empty string `""` |
| `get_string_or("KEY", "default")` | Get with default | `"default"` |
| `get_u16("KEY")` | Get unsigned 16-bit int | `0` |
| `get_u16_or("KEY", 3000)` | Get with default | `3000` |
| `get_u32("KEY")` | Get unsigned 32-bit int | `0` |
| `get_u32_or("KEY", 3000)` | Get with default | `3000` |
| `get_i32("KEY")` | Get signed 32-bit int | `0` |
| `get_i32_or("KEY", -1)` | Get with default | `-1` |
| `get_bool("KEY")` | Get boolean | `false` |
| `get_bool_or("KEY", true)` | Get with default | `true` |
| `get("KEY")` | Get as `Option<&str>` | `None` |
| `has("KEY")` | Check if key exists | `false` |

## Environment File

By default, `ConfigModule::for_root().env_file(".env")` loads from a `.env` file in your project root.

Create a `.env` file:

```bash
APP_NAME=My NestForge App
PORT=3000
DEBUG=true
DATABASE_URL=postgres://user:pass@localhost/mydb
```

## Multiple Config Files

For feature-specific configs, use `ConfigModule::for_feature()`:

```rust
// src/config/database.rs
use nestforge_config::{ConfigService, ConfigModule};

pub type DatabaseConfig = ConfigService;

pub fn load_database_config() -> DatabaseConfig {
    ConfigModule::for_root_with_options(
        ConfigModule::for_feature().env_file(".env.database")
    )
}
```

## Typed Configs

For strongly-typed configuration structures, use `register_config`:

```rust
use nestforge_config::register_config;

#[derive(Debug, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub static DATABASE_CONFIG = register_config("database", || DatabaseSettings {
    host: "localhost".to_string(),
    port: 5432,
    username: "postgres".to_string(),
    password: "password".to_string(),
});

// Usage
let db = DATABASE_CONFIG.load();
println!("Host: {}", db.host);
```

## Options

### Custom .env Path

```rust
ConfigModule::for_root_with_options(
    ConfigModule::for_root().env_file(".env.production")
)
```

### Skip Process Environment

```rust
ConfigModule::for_root_with_options(
    ConfigModule::for_root().without_process_env()
)
```

## Example .env File

```bash
# Application
APP_NAME=NestForge Application
PORT=3000
HOST=127.0.0.1

# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/mydb

# Features
DEBUG=true
ENABLE_CORS=true

# API Keys
API_KEY=your-api-key-here
```

## NestJS Comparison

If you're coming from NestJS, here's how our API compares:

| NestJS | NestForge |
|--------|-----------|
| `ConfigModule.forRoot({ isGlobal: true })` | `ConfigModule::for_root()` |
| `configService.get('KEY')` | `config.get_string("KEY")` |
| `configService.getOrThrow('KEY')` | `config.get("KEY").unwrap()` |
| `registerAs('config', () => ({ ... }))` | `register_config("name", \|\| Config { ... })` |
| `@Inject(ConfigService)` | `Config<AppConfig>` |

## Error Handling

If loading the `.env` file fails, the application will panic with a clear error message:

```rust
pub fn load_config() -> AppConfig {
    ConfigModule::for_root_with_options(ConfigModule::for_root().env_file(".env"))
    // Panics if .env file cannot be read
}
```

For custom error handling:

```rust
pub fn load_config() -> Result<AppConfig, ConfigError> {
    ConfigModule::for_root_with_options(ConfigModule::for_root().env_file(".env"))
}
```
