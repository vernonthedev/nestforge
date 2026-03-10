#![cfg(feature = "graphql")]

use nestforge::{
    async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema},
    graphql_auth_identity, graphql_request_id, graphql_router_with_config, resolve_graphql,
    AuthIdentity, Container, GraphQlConfig, NestForgeFactory, NestForgeFactoryGraphQlExt, Provider,
    RequestContext,
};
use tower::ServiceExt;

#[derive(Clone)]
struct AppConfig {
    app_name: &'static str,
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self, ctx: &Context<'_>) -> &str {
        let config = resolve_graphql::<AppConfig>(ctx).expect("app config should resolve");

        config.app_name
    }
}

#[tokio::test]
async fn graphql_router_accepts_post_requests() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let container = Container::new();
    container
        .register(AppConfig { app_name: "ok" })
        .expect("app config should register");
    let app = graphql_router_with_config(schema, GraphQlConfig::new("/graphql").without_graphiql())
        .with_state(container);

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/graphql")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({ "query": "{ health }" }).to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn graphql_router_rejects_requests_above_max_body_size() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let container = Container::new();
    container
        .register(AppConfig { app_name: "ok" })
        .expect("app config should register");
    let app = graphql_router_with_config(
        schema,
        GraphQlConfig::new("/graphql")
            .without_graphiql()
            .with_max_request_bytes(32),
    )
    .with_state(container);
    let payload = serde_json::json!({
        "query": "{ health }",
        "variables": {
            "payload": "this request body is intentionally too large"
        }
    })
    .to_string();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/graphql")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .header(axum::http::header::CONTENT_LENGTH, payload.len())
                .body(axum::body::Body::from(payload))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::PAYLOAD_TOO_LARGE);
}

#[derive(Clone)]
struct ScopedRequestInfo {
    path: String,
}

struct ScopedModule;

impl nestforge::ModuleDefinition for ScopedModule {
    fn register(container: &Container) -> anyhow::Result<()> {
        nestforge::register_provider(
            container,
            Provider::request_factory(|container| {
                let ctx = container.resolve::<RequestContext>()?;
                Ok(ScopedRequestInfo {
                    path: ctx.uri.path().to_string(),
                })
            }),
        )?;
        Ok(())
    }
}

struct ScopedQueryRoot;

#[Object]
impl ScopedQueryRoot {
    async fn request_id(&self, ctx: &Context<'_>) -> String {
        graphql_request_id(ctx)
            .expect("request id should be present")
            .value()
            .to_string()
    }

    async fn subject(&self, ctx: &Context<'_>) -> Option<String> {
        graphql_auth_identity(ctx).map(|identity| identity.subject.clone())
    }

    async fn scoped_path(&self, ctx: &Context<'_>) -> String {
        let info = resolve_graphql::<ScopedRequestInfo>(ctx).expect("scoped info should resolve");
        info.path.clone()
    }
}

#[tokio::test]
async fn graphql_router_uses_scoped_container_and_request_metadata_under_factory() {
    let schema = Schema::build(ScopedQueryRoot, EmptyMutation, EmptySubscription).finish();
    let app = NestForgeFactory::<ScopedModule>::create()
        .expect("factory should build")
        .with_auth_resolver(|token, _container| async move {
            Ok(token.map(|_| AuthIdentity::new("graphql-user")))
        })
        .with_graphql(schema)
        .into_router();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/graphql")
                .header(axum::http::header::AUTHORIZATION, "Bearer demo-token")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({
                        "query": "{ requestId subject scopedPath }"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
