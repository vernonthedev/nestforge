use std::{marker::PhantomData, sync::Arc};

use anyhow::Result;
use async_graphql::{ObjectType, Schema, SubscriptionType};
use axum::Router;
use nestforge_core::{
    initialize_module_runtime, Container, ContainerError, InitializedModule, ModuleDefinition,
};
use nestforge_graphql::{graphql_router, graphql_router_with_config, GraphQlConfig};

type OverrideFn = Box<dyn Fn(&Container) -> Result<()> + Send + Sync>;

pub struct TestFactory<M: ModuleDefinition> {
    overrides: Vec<OverrideFn>,
    _marker: PhantomData<M>,
}

impl<M: ModuleDefinition> TestFactory<M> {
    pub fn create() -> Self {
        Self {
            overrides: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn override_provider<T>(mut self, value: T) -> Self
    where
        T: Send + Sync + Clone + 'static,
    {
        self.overrides.push(Box::new(move |container| {
            container.override_value(value.clone())?;
            Ok(())
        }));
        self
    }

    pub fn build(self) -> Result<TestingModule> {
        let container = Container::new();

        for override_fn in self.overrides {
            override_fn(&container)?;
        }

        let runtime = initialize_module_runtime::<M>(&container)?;
        runtime.run_module_init(&container)?;
        runtime.run_application_bootstrap(&container)?;

        Ok(TestingModule {
            container,
            runtime: Arc::new(runtime),
        })
    }
}

#[derive(Clone)]
pub struct TestingModule {
    container: Container,
    runtime: Arc<InitializedModule>,
}

impl TestingModule {
    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn resolve<T>(&self) -> Result<Arc<T>, ContainerError>
    where
        T: Send + Sync + 'static,
    {
        self.container.resolve::<T>()
    }

    pub fn http_router(&self) -> Router {
        let mut app: Router<Container> = Router::new();
        for controller_router in &self.runtime.controllers {
            app = app.merge(controller_router.clone());
        }

        app.with_state(self.container.clone())
    }

    pub fn graphql_router<Query, Mutation, Subscription>(
        &self,
        schema: Schema<Query, Mutation, Subscription>,
    ) -> Router
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        self.http_router()
            .merge(graphql_router(schema).with_state(self.container.clone()))
    }

    pub fn graphql_router_with_paths<Query, Mutation, Subscription>(
        &self,
        schema: Schema<Query, Mutation, Subscription>,
        endpoint: impl Into<String>,
        graphiql_endpoint: Option<String>,
    ) -> Router
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let config = if let Some(graphiql_endpoint) = graphiql_endpoint {
            GraphQlConfig::new(endpoint).with_graphiql(graphiql_endpoint)
        } else {
            GraphQlConfig::new(endpoint).without_graphiql()
        };

        self.http_router()
            .merge(graphql_router_with_config(schema, config).with_state(self.container.clone()))
    }

    pub fn shutdown(&self) -> Result<()> {
        self.runtime.run_module_destroy(&self.container)?;
        self.runtime.run_application_shutdown(&self.container)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc as StdArc, Mutex};

    use async_graphql::{EmptyMutation, EmptySubscription};
    use nestforge_core::{register_provider, ControllerDefinition, LifecycleHook, Provider};
    use tower::ServiceExt;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct AppConfig {
        app_name: &'static str,
    }

    struct AppModule;
    impl ModuleDefinition for AppModule {
        fn register(container: &Container) -> Result<()> {
            container.register(AppConfig {
                app_name: "default",
            })?;
            Ok(())
        }
    }

    #[test]
    fn builds_testing_module_and_resolves_default_provider() {
        let module = TestFactory::<AppModule>::create()
            .build()
            .expect("test module should build");

        let config = module
            .resolve::<AppConfig>()
            .expect("config should resolve");
        assert_eq!(
            *config,
            AppConfig {
                app_name: "default"
            }
        );
    }

    #[test]
    fn overrides_provider_value() {
        let module = TestFactory::<AppModule>::create()
            .override_provider(AppConfig { app_name: "test" })
            .build()
            .expect("test module should build with overrides");

        let config = module
            .resolve::<AppConfig>()
            .expect("config should resolve");
        assert_eq!(*config, AppConfig { app_name: "test" });
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct GreetingService {
        greeting: String,
    }

    struct FactoryModule;
    impl ModuleDefinition for FactoryModule {
        fn register(container: &Container) -> Result<()> {
            register_provider(
                container,
                Provider::factory(|container| {
                    let config = container.resolve::<AppConfig>()?;
                    Ok(GreetingService {
                        greeting: format!("hello {}", config.app_name),
                    })
                }),
            )?;
            Ok(())
        }
    }

    #[test]
    fn overrides_are_applied_before_factory_resolution() {
        let module = TestFactory::<FactoryModule>::create()
            .override_provider(AppConfig {
                app_name: "override",
            })
            .build()
            .expect("test module should build with transitive overrides");

        let greeting = module
            .resolve::<GreetingService>()
            .expect("greeting service should resolve");
        assert_eq!(greeting.greeting, "hello override");
    }

    struct HttpController;

    impl ControllerDefinition for HttpController {
        fn router() -> Router<Container> {
            Router::new().route(
                "/health",
                axum::routing::get(|| async { axum::Json(serde_json::json!({ "ok": true })) }),
            )
        }
    }

    struct HttpModule;
    impl ModuleDefinition for HttpModule {
        fn register(_container: &Container) -> Result<()> {
            Ok(())
        }

        fn controllers() -> Vec<Router<Container>> {
            vec![HttpController::router()]
        }
    }

    #[tokio::test]
    async fn builds_http_router_from_testing_module_runtime() {
        let module = TestFactory::<HttpModule>::create()
            .build()
            .expect("http testing module should build");

        let response = module
            .http_router()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    struct QueryRoot;

    #[async_graphql::Object]
    impl QueryRoot {
        async fn app_name(&self, ctx: &async_graphql::Context<'_>) -> &str {
            let config = ctx
                .data::<Container>()
                .expect("container should be present")
                .resolve::<AppConfig>()
                .expect("app config should resolve");

            config.app_name
        }
    }

    #[tokio::test]
    async fn builds_graphql_router_from_testing_module_runtime() {
        let module = TestFactory::<AppModule>::create()
            .override_provider(AppConfig { app_name: "graphql" })
            .build()
            .expect("graphql testing module should build");
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

        let response = module
            .graphql_router_with_paths(schema, "/graphql", None)
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({ "query": "{ appName }" }).to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("graphql request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[derive(Clone)]
    struct HookLog(StdArc<Mutex<Vec<&'static str>>>);

    fn record_destroy(container: &Container) -> Result<()> {
        let log = container.resolve::<HookLog>()?;
        log.0.lock().expect("hook log should be writable").push("destroy");
        Ok(())
    }

    fn record_shutdown(container: &Container) -> Result<()> {
        let log = container.resolve::<HookLog>()?;
        log.0
            .lock()
            .expect("hook log should be writable")
            .push("shutdown");
        Ok(())
    }

    struct HookModule;

    impl ModuleDefinition for HookModule {
        fn register(container: &Container) -> Result<()> {
            container.register(HookLog(StdArc::new(Mutex::new(Vec::new()))))?;
            Ok(())
        }

        fn on_module_destroy() -> Vec<LifecycleHook> {
            vec![record_destroy]
        }

        fn on_application_shutdown() -> Vec<LifecycleHook> {
            vec![record_shutdown]
        }
    }

    #[test]
    fn shutdown_runs_destroy_and_shutdown_hooks() {
        let module = TestFactory::<HookModule>::create()
            .build()
            .expect("hook testing module should build");

        module.shutdown().expect("testing module should shut down");

        let log = module.resolve::<HookLog>().expect("hook log should resolve");
        let entries = log.0.lock().expect("hook log should be readable").clone();
        assert_eq!(entries, vec!["destroy", "shutdown"]);
    }
}
