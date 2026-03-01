#![cfg(feature = "graphql")]

use nestforge::{
    async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema},
    Container,
    graphql_router_with_config, GraphQlConfig,
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
        let container = ctx
            .data::<Container>()
            .expect("container should be present in graphql context");
        let config = container
            .resolve::<AppConfig>()
            .expect("app config should resolve");

        config.app_name
    }
}

#[tokio::test]
async fn graphql_router_accepts_post_requests() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let container = Container::new();
    container
        .register(AppConfig {
            app_name: "ok",
        })
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
