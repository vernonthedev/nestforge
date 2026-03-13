use nestforge::{injectable, MicroserviceRegistry};

#[injectable(factory = build_grpc_patterns)]
pub struct GrpcPatterns {
    registry: MicroserviceRegistry,
}

fn build_grpc_patterns() -> GrpcPatterns {
    GrpcPatterns {
        registry: MicroserviceRegistry::builder()
            .message("hello.say", |name: String, ctx| async move {
                let config = ctx.resolve::<crate::AppConfig>()?;
                Ok(serde_json::json!({
                    "message": format!("Hello, {name}! Welcome to {}.", config.app_name),
                }))
            })
            .build(),
    }
}

impl GrpcPatterns {
    pub fn registry(&self) -> &MicroserviceRegistry {
        &self.registry
    }
}
