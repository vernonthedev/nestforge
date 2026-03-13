use nestforge::{controller, injectable, module, routes, HttpException, Inject, NestForgeFactory};
use tower::ServiceExt;

#[injectable]
struct DefaultGreetingService {
    label: &'static str,
}

impl Default for DefaultGreetingService {
    fn default() -> Self {
        Self { label: "hello" }
    }
}

#[injectable(factory = build_factory_service)]
struct FactoryGreetingService {
    label: &'static str,
}

fn build_factory_service() -> anyhow::Result<FactoryGreetingService> {
    Ok(FactoryGreetingService { label: "factory" })
}

#[controller("/injectable")]
#[derive(Default)]
struct InjectableController;

#[routes]
impl InjectableController {
    #[nestforge::get("/default")]
    async fn default(service: Inject<DefaultGreetingService>) -> Result<String, HttpException> {
        let clone = (*service).clone();
        Ok(clone.label.to_string())
    }

    #[nestforge::get("/factory")]
    async fn factory(service: Inject<FactoryGreetingService>) -> Result<String, HttpException> {
        let clone = (*service).clone();
        Ok(clone.label.to_string())
    }
}

#[module(
    controllers = [InjectableController],
    providers = [DefaultGreetingService, FactoryGreetingService],
    exports = [DefaultGreetingService, FactoryGreetingService]
)]
struct InjectableModule;

#[tokio::test]
async fn injectable_default_registration_resolves_from_module_container() {
    let factory = NestForgeFactory::<InjectableModule>::create().expect("factory should build");

    let service = factory
        .container()
        .resolve::<DefaultGreetingService>()
        .expect("default injectable should resolve");

    assert_eq!(service.label, "hello");
    assert_eq!((*service).clone().label, "hello");
}

#[tokio::test]
async fn injectable_factory_registration_flows_through_http_injection() {
    let app = NestForgeFactory::<InjectableModule>::create()
        .expect("factory should build")
        .into_router();

    let default_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/injectable/default")
                .body(axum::body::Body::empty())
                .expect("default request should build"),
        )
        .await
        .expect("default request should succeed");
    let default_body = axum::body::to_bytes(default_response.into_body(), usize::MAX)
        .await
        .expect("default response body");

    let factory_response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/injectable/factory")
                .body(axum::body::Body::empty())
                .expect("factory request should build"),
        )
        .await
        .expect("factory request should succeed");
    let factory_body = axum::body::to_bytes(factory_response.into_body(), usize::MAX)
        .await
        .expect("factory response body");

    assert_eq!(std::str::from_utf8(&default_body).expect("utf8"), "\"hello\"");
    assert_eq!(std::str::from_utf8(&factory_body).expect("utf8"), "\"factory\"");
}
