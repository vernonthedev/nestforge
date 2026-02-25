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
cargo run -p nestforge-cli -- new demo-api
```

Then move into the app and run it with Cargo.
