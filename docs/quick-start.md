# Quick Start

## Requirements

- Rust and Cargo installed

## Run The Workspace

```bash
cargo check --workspace
cargo test --workspace
cargo run -p hello-nestforge
```

Server URL:

```text
http://127.0.0.1:3000
```

## Install CLI

```bash
cargo install --path crates/nestforge-cli
```

## Create A New App

```bash
nestforge new demo-api
cd demo-api
cargo run
```

## Generate Code

```bash
nestforge g module users
nestforge g resource users --module users
nestforge g guard auth
nestforge g interceptor logging
```

## Run DB Commands

```bash
nestforge db init
nestforge db generate create_users_table
nestforge db migrate
nestforge db status
```

## Useful Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -D warnings
nestforge docs
nestforge fmt
```
