#![cfg(feature = "graphql")]

use nestforge::{
    async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema},
    graphql_router_with_config, GraphQlConfig,
};
use tower::ServiceExt;

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self, _ctx: &Context<'_>) -> &str {
        "ok"
    }
}

#[tokio::test]
async fn graphql_router_accepts_post_requests() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let app = graphql_router_with_config(schema, GraphQlConfig::new("/graphql").without_graphiql());

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
