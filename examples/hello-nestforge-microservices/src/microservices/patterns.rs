use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct EventCounter(pub Arc<AtomicUsize>);

#[derive(Clone)]
pub struct AppPatterns {
    registry: nestforge::MicroserviceRegistry,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub name: String,
}

impl AppPatterns {
    pub fn new() -> Self {
        Self {
            registry: nestforge::MicroserviceRegistry::builder()
                .message("app.greet", |payload: GreetingPayload, ctx| async move {
                    let config = ctx.resolve::<crate::app_config::AppConfig>()?;
                    Ok(serde_json::json!({
                        "message": format!("Hello, {}! Welcome to {}.", payload.name, config.app_name),
                        "transport": ctx.transport(),
                    }))
                })
                .event("app.bump", |_payload: (), ctx| async move {
                    let counter = ctx.resolve::<EventCounter>()?;
                    counter.0.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .build(),
        }
    }

    pub fn registry(&self) -> &nestforge::MicroserviceRegistry {
        &self.registry
    }
}
