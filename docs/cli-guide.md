# CLI Guide

## Run CLI

```bash
nestforge <command>
```

## Commands

- `new <app-name>`: create a new app in your current directory
- `g resource <name>`: generate controller + service + DTOs
- `g controller <name>`: generate controller only
- `g service <name>`: generate service only

## Notes

- Install CLI once with `cargo install nestforge-cli`.
- Generator commands must run inside an app folder (`Cargo.toml` + `src/` present).
- The CLI patches `mod.rs` and `app_module.rs` so new files are wired in.
