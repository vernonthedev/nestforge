use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use nestforge::injectable;

use crate::AppConfig;

#[injectable(factory = build_event_counter)]
pub struct EventCounter(pub Arc<AtomicUsize>);

#[injectable(factory = build_app_patterns)]
pub struct AppPatterns {
    registry: nestforge::MicroserviceRegistry,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GreetingPayload {
    pub name: String,
}

fn build_event_counter() -> EventCounter {
    EventCounter(Arc::new(AtomicUsize::new(0)))
}

fn build_app_patterns() -> AppPatterns {
    AppPatterns {
        registry: nestforge::MicroserviceRegistry::builder()
            .message("app.greet", |payload: GreetingPayload, ctx| async move {
                let config = ctx.resolve::<AppConfig>()?;
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

impl AppPatterns {
    pub fn registry(&self) -> &nestforge::MicroserviceRegistry {
        &self.registry
    }
}
