use nestforge::MicroserviceRegistry;

#[derive(Clone)]
pub struct WsPatterns {
    registry: MicroserviceRegistry,
}

impl WsPatterns {
    pub fn new() -> Self {
        Self {
            registry: MicroserviceRegistry::builder()
                .message("app.info", |_payload: (), ctx| async move {
                    let config = ctx.resolve::<crate::app_config::AppConfig>()?;
                    Ok(serde_json::json!({
                        "app_name": config.app_name,
                        "transport": ctx.transport(),
                    }))
                })
                .event("app.ping", |_payload: serde_json::Value, _ctx| async move {
                    Ok(())
                })
                .build(),
        }
    }

    pub fn registry(&self) -> &MicroserviceRegistry {
        &self.registry
    }
}
