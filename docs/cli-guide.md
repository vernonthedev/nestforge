# CLI Guide

## Run CLI

```bash
cargo run -p nestforge-cli -- <command>
```

## Commands

- `new <app-name>`: create a new app inside `examples/<app-name>`
- `g resource <name>`: generate controller + service + DTOs
- `g controller <name>`: generate controller only
- `g service <name>`: generate service only

## Notes

- Generator commands must run inside an app folder (`Cargo.toml` + `src/` present).
- The CLI patches `mod.rs` and `app_module.rs` so new files are wired in.
