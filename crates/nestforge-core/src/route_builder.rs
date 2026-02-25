use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{Container, ControllerBasePath};

/*
RouteBuilder<T> helps us build routes cleanly in generated code.
It prefixes method routes using the controller base path from #[controller("...")].
*/
pub struct RouteBuilder<T> {
    router: Router<Container>,
    _marker: std::marker::PhantomData<T>,
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
        let full = Self::full_path(path);

        Self {
            router: self.router.route(&full, get(handler)),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn post<H, TState>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        let full = Self::full_path(path);

        Self {
            router: self.router.route(&full, post(handler)),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn put<H, TState>(self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<TState, Container> + Clone + Send + Sync + 'static,
        TState: 'static,
    {
        let full = Self::full_path(path);

        Self {
            router: self.router.route(&full, put(handler)),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn build(self) -> Router<Container> {
        self.router
    }
}