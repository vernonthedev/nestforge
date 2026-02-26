use axum::{
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};

use std::sync::Arc;

use crate::{execute_pipeline, framework_log, Container, ControllerBasePath, Guard, Interceptor};

/*
RouteBuilder<T> helps us build routes cleanly in generated code.
It prefixes method routes using the controller base path from #[controller("...")].
*/
pub struct RouteBuilder<T> {
    router: Router<Container>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for RouteBuilder<T>
where
    T: ControllerBasePath,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> RouteBuilder<T>
where
    T: ControllerBasePath,
{
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            _marker: std::marker::PhantomData,
        }
    }

    fn full_path(path: &str) -> String {
        let base = T::base_path().trim_end_matches('/');
        let path = path.trim();

        let sub = if path == "/" {
            ""
        } else {
            path.trim_start_matches('/')
        };

        if base.is_empty() {
            if sub.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", sub)
            }
        } else if sub.is_empty() {
            base.to_string()
        } else {
            format!("{}/{}", base, sub)
        }
    }

    pub fn get<H, TState>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.get_with_pipeline(path, handler, Vec::new(), Vec::new())
    }

    pub fn post<H, TState>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.post_with_pipeline(path, handler, Vec::new(), Vec::new())
    }

    pub fn put<H, TState>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.put_with_pipeline(path, handler, Vec::new(), Vec::new())
    }

    pub fn delete<H, TState>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.delete_with_pipeline(path, handler, Vec::new(), Vec::new())
    }

    pub fn get_with_pipeline<H, TState>(
        self,
        path: &str,
        handler: H,
        guards: Vec<Arc<dyn Guard>>,
        interceptors: Vec<Arc<dyn Interceptor>>,
    ) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.route_with_pipeline("GET", path, get(handler), guards, interceptors)
    }

    pub fn post_with_pipeline<H, TState>(
        self,
        path: &str,
        handler: H,
        guards: Vec<Arc<dyn Guard>>,
        interceptors: Vec<Arc<dyn Interceptor>>,
    ) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.route_with_pipeline("POST", path, post(handler), guards, interceptors)
    }

    pub fn put_with_pipeline<H, TState>(
        self,
        path: &str,
        handler: H,
        guards: Vec<Arc<dyn Guard>>,
        interceptors: Vec<Arc<dyn Interceptor>>,
    ) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.route_with_pipeline("PUT", path, put(handler), guards, interceptors)
    }

    pub fn delete_with_pipeline<H, TState>(
        self,
        path: &str,
        handler: H,
        guards: Vec<Arc<dyn Guard>>,
        interceptors: Vec<Arc<dyn Interceptor>>,
    ) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        self.route_with_pipeline("DELETE", path, delete(handler), guards, interceptors)
    }

    fn route_with_pipeline(
        self,
        method: &str,
        path: &str,
        method_router: axum::routing::MethodRouter<Container>,
        guards: Vec<Arc<dyn Guard>>,
        interceptors: Vec<Arc<dyn Interceptor>>,
    ) -> Self {
        let full = Self::full_path(path);
        framework_log(format!("Registering router '{} {}'.", method, full));
        let guards = Arc::new(guards);
        let interceptors = Arc::new(interceptors);

        let route = method_router.route_layer(from_fn(move |req, next| {
            let guards = Arc::clone(&guards);
            let interceptors = Arc::clone(&interceptors);
            async move { execute_pipeline(req, next, guards, interceptors).await }
        }));

        Self {
            router: self.router.route(&full, route),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn build(self) -> Router<Container> {
        self.router
    }
}
