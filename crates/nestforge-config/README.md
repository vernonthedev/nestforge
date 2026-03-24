# nestforge-config

A NestJS-inspired configuration module for NestForge applications.

## Features

- **Type-safe configuration** - Get typed values from environment variables
- **NestJS-like API** - Familiar patterns for NestJS developers
- **DI Integration** - Seamlessly works with NestForge's dependency injection
- **Simple getters** - No unwrap_or chains, just `get_string_or("KEY", "default")`
- **Dotenv support** - Automatic loading from `.env` files

## Quick Start

```rust
use nestforge_config::{ConfigService, ConfigModule};

pub type AppConfig = ConfigService;

pub fn load_config() -> AppConfig {
    ConfigModule::for_root_with_options(ConfigModule::for_root().env_file(".env"))
}
```

## Usage in Services

```rust
use nestforge::prelude::*;

pub struct MyService {
    config: Config<AppConfig>,
}

impl MyService {
    pub fn do_something(&self) {
        let app_name = self.config.get_string_or("APP_NAME", "My App");
        let port = self.config.get_u16_or("PORT", 3000);
        let debug = self.config.get_bool_or("DEBUG", false);
    }
}
```

## Available Methods

| Method | Description |
|--------|-------------|
| `get_string("KEY")` | Get string (default: `""`) |
| `get_string_or("KEY", "default")` | Get with default |
| `get_u16("KEY")` | Get u16 (default: `0`) |
| `get_u16_or("KEY", 3000)` | Get with default |
| `get_bool("KEY")` | Get bool (default: `false`) |
| `get_bool_or("KEY", true)` | Get with default |
| `get("KEY")` | Get `Option<&str>` |
| `has("KEY")` | Check if key exists |

## Resources

- [Documentation](https://github.com/vernonthedev/nestforge/wiki)
- [Examples](https://github.com/vernonthedev/nestforge/tree/main/examples)
- [Discord Community](https://discord.gg/nestforge)
