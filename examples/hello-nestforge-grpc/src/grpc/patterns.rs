use nestforge::MicroserviceRegistry;

#[derive(Clone)]
pub struct GrpcPatterns {
    registry: MicroserviceRegistry,
}

impl GrpcPatterns {
    pub fn new() -> Self {
        Self {
            registry: MicroserviceRegistry::builder()
                .message("hello.say", |name: String, ctx| async move {
                    let config = ctx.resolve::<crate::app_config::AppConfig>()?;
                    Ok(serde_json::json!({
                        "message": format!("Hello, {name}! Welcome to {}.", config.app_name),
                    }))
                })
                .build(),
        }
    }

    pub fn registry(&self) -> &MicroserviceRegistry {
        &self.registry
    }
}
