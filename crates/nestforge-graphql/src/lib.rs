use std::{any::type_name, sync::Arc};

use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, ObjectType, Schema, SubscriptionType,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{DefaultBodyLimit, Extension, FromRequest, Request, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    RequestExt, Router,
};
use nestforge_core::{AuthIdentity, Container, RequestId};

pub use async_graphql;

pub type GraphQlSchema<Query, Mutation = EmptyMutation, Subscription = EmptySubscription> =
    Schema<Query, Mutation, Subscription>;

pub fn graphql_container<'ctx>(
    ctx: &'ctx async_graphql::Context<'ctx>,
) -> async_graphql::Result<&'ctx Container> {
    ctx.data::<Container>()
}

pub fn graphql_request_id<'ctx>(
    ctx: &'ctx async_graphql::Context<'ctx>,
) -> Option<&'ctx RequestId> {
    ctx.data_opt::<RequestId>()
}

pub fn graphql_auth_identity<'ctx>(
    ctx: &'ctx async_graphql::Context<'ctx>,
) -> Option<&'ctx AuthIdentity> {
    ctx.data_opt::<Arc<AuthIdentity>>().map(AsRef::as_ref)
}

pub fn resolve_graphql<T>(ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Arc<T>>
where
    T: Send + Sync + 'static,
{
    let container = graphql_container(ctx)?;
    container.resolve::<T>().map_err(|_| {
        async_graphql::Error::new(format!(
            "Failed to resolve dependency `{}` from GraphQL context",
            type_name::<T>()
        ))
    })
}

#[derive(Debug, Clone)]
pub struct GraphQlConfig {
    pub endpoint: String,
    pub graphiql_endpoint: Option<String>,
    pub max_request_bytes: usize,
}

impl Default for GraphQlConfig {
    fn default() -> Self {
        Self {
            endpoint: "/graphql".to_string(),
            graphiql_endpoint: Some("/graphiql".to_string()),
            max_request_bytes: 1024 * 1024,
        }
    }
}

impl GraphQlConfig {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: normalize_path(endpoint.into()),
            ..Self::default()
        }
    }

    pub fn with_graphiql(mut self, path: impl Into<String>) -> Self {
        self.graphiql_endpoint = Some(normalize_path(path.into()));
        self
    }

    pub fn without_graphiql(mut self) -> Self {
        self.graphiql_endpoint = None;
        self
    }

    pub fn with_max_request_bytes(mut self, bytes: usize) -> Self {
        self.max_request_bytes = bytes;
        self
    }
}

pub fn graphql_router<Query, Mutation, Subscription>(
    schema: GraphQlSchema<Query, Mutation, Subscription>,
) -> Router<Container>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    graphql_router_with_config(schema, GraphQlConfig::default())
}

pub fn graphql_router_with_config<Query, Mutation, Subscription>(
    schema: GraphQlSchema<Query, Mutation, Subscription>,
    config: GraphQlConfig,
) -> Router<Container>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    let max_request_bytes = config.max_request_bytes;
    let mut router = Router::new()
        .route(
            &config.endpoint,
            post(
                move |container, scoped_container, request_id, auth_identity, schema, request| {
                    graphql_handler::<Query, Mutation, Subscription>(
                        max_request_bytes,
                        container,
                        scoped_container,
                        request_id,
                        auth_identity,
                        schema,
                        request,
                    )
                },
            ),
        )
        .layer(DefaultBodyLimit::max(config.max_request_bytes))
        .layer(Extension(schema));

    if let Some(graphiql_endpoint) = &config.graphiql_endpoint {
        let endpoint = config.endpoint.clone();
        let graphiql_html = GraphiQLSource::build().endpoint(&endpoint).finish();
        router = router.route(
            graphiql_endpoint,
            get(move || {
                let html = graphiql_html.clone();
                async move { Html(html) }
            }),
        );
    }

    router
}

async fn graphql_handler<Query, Mutation, Subscription>(
    max_request_bytes: usize,
    State(container): State<Container>,
    scoped_container: Option<Extension<Container>>,
    request_id: Option<Extension<RequestId>>,
    auth_identity: Option<Extension<Arc<AuthIdentity>>>,
    Extension(schema): Extension<GraphQlSchema<Query, Mutation, Subscription>>,
    request: Request,
) -> Response
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    if request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > max_request_bytes)
    {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }

    let request =
        match GraphQLRequest::<async_graphql_axum::rejection::GraphQLRejection>::from_request(
            request.with_limited_body(),
            &(),
        )
        .await
        {
            Ok(request) => request,
            Err(rejection) => return rejection.into_response(),
        };

    let container = scoped_container.map(|value| value.0).unwrap_or(container);
    let mut request = request.into_inner().data(container);
    if let Some(Extension(request_id)) = request_id {
        request = request.data(request_id);
    }
    if let Some(Extension(auth_identity)) = auth_identity {
        request = request.data(auth_identity);
    }

    GraphQLResponse::from(schema.execute(request).await).into_response()
}

fn normalize_path(path: String) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/graphql".to_string();
    }

    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}
