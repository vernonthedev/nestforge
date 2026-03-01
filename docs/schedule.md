# Schedule

NestForge includes optional scheduler support through the `nestforge-schedule` crate.

## Enable The Feature

```toml
nestforge = { version = "1", features = ["schedule"] }
```

## Core Pieces

- `ScheduleRegistry`
- `ScheduleRegistryBuilder`
- `start_schedules(container)`
- `shutdown_schedules(container)`

## Registering Jobs

Add `ScheduleRegistry` as a provider, register jobs, and start them with module lifecycle hooks:

```rust
fn build_schedule_registry() -> anyhow::Result<nestforge::ScheduleRegistry> {
    Ok(
        nestforge::ScheduleRegistry::builder()
            .every_named("metrics", std::time::Duration::from_secs(30), || async {
                println!("running periodic job");
            })
            .after_named("warmup", std::time::Duration::from_secs(5), || async {
                println!("warming cache");
            })
            .build(),
    )
}

#[module(
    providers = [build_schedule_registry()?],
    on_application_bootstrap = [nestforge::start_schedules],
    on_application_shutdown = [nestforge::shutdown_schedules]
)]
pub struct AppModule;
```

## Supported Job Types

- `registry.every(duration, task)` for repeated work
- `registry.after(duration, task)` for one-shot delayed work
- `registry.every_named(name, duration, task)` for named repeated work
- `registry.after_named(name, duration, task)` for named delayed work

## Introspection

You can inspect configured jobs before startup:

```rust
let jobs = registry.jobs();
```

Each `ScheduledJob` includes:

- `name`
- `kind`
- `delay`
