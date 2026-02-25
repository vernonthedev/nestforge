# Quick Start

## Requirements

- Rust + Cargo installed

## Run checks

```bash
cargo check
```

## Run example app

```bash
cargo run -p hello-nestforge
```

Server starts at:

```text
http://127.0.0.1:3000
```

## Useful commands

```bash
cargo fmt
cargo clippy --workspace --all-targets --all-features
cargo test
```

## Create a new app with CLI

```bash
cargo install nestforge-cli
nestforge new demo-api
cd demo-api
cargo run
```

## Add NestForge to an existing project

```toml
[dependencies]
nestforge = "1.0.0"
```
