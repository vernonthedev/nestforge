use std::{any::type_name, sync::Arc};

use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, ObjectType, Schema, SubscriptionType,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{Extension, State},
    response::Html,
    routing::{get, post},
    Router,
};
use nestforge_core::{AuthIdentity, Container, RequestId};

pub use async_graphql;

pub type GraphQlSchema<Query, Mutation = EmptyMutation, Subscription = EmptySubscription> =
    Schema<Query, Mutation, Subscription>;

pub fn graphql_container(ctx: &async_graphql::Context<'_>) -> async_graphql::Result<&Container> {
    ctx.data::<Container>()
}

pub fn graphql_request_id(ctx: &async_graphql::Context<'_>) -> Option<&RequestId> {
    ctx.data_opt::<RequestId>()
}

pub fn graphql_auth_identity(ctx: &async_graphql::Context<'_>) -> Option<&AuthIdentity> {
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
}

impl Default for GraphQlConfig {
    fn default() -> Self {
        Self {
            endpoint: "/graphql".to_string(),
            graphiql_endpoint: Some("/graphiql".to_string()),
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
    let mut router = Router::new()
        .route(&config.endpoint, post(graphql_handler::<Query, Mutation, Subscription>))
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
    State(container): State<Container>,
    scoped_container: Option<Extension<Container>>,
    request_id: Option<RequestId>,
    auth_identity: Option<Extension<Arc<AuthIdentity>>>,
    Extension(schema): Extension<GraphQlSchema<Query, Mutation, Subscription>>,
    request: GraphQLRequest,
) -> GraphQLResponse
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    let container = scoped_container.map(|value| value.0).unwrap_or(container);
    let mut request = request.into_inner().data(container);
    if let Some(request_id) = request_id {
        request = request.data(request_id);
    }
    if let Some(Extension(auth_identity)) = auth_identity {
        request = request.data(auth_identity);
    }

    schema.execute(request).await.into()
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
