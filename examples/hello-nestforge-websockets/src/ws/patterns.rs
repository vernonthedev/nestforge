use nestforge::{injectable, MicroserviceRegistry};

#[injectable(factory = build_ws_patterns)]
pub struct WsPatterns {
    registry: MicroserviceRegistry,
}

fn build_ws_patterns() -> WsPatterns {
    WsPatterns {
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

impl WsPatterns {
    pub fn registry(&self) -> &MicroserviceRegistry {
        &self.registry
    }
}
