use std::sync::Arc;

use axum::{body::Body, extract::Request, http::Method};
use nestforge_core::{framework_log_event, NextFn, NextFuture};

pub trait NestMiddleware: Send + Sync + 'static {
    fn handle(&self, req: Request<Body>, next: NextFn) -> NextFuture;
}

#[derive(Clone)]
pub struct MiddlewareBinding {
    middleware: Arc<dyn NestMiddleware>,
    matcher: RouteMatcher,
}

impl MiddlewareBinding {
    fn matches(&self, method: &Method, path: &str) -> bool {
        self.matcher.matches(method, path)
    }
}

#[derive(Clone, Default)]
struct RouteMatcher {
    include: Vec<MiddlewareRoute>,
    exclude: Vec<MiddlewareRoute>,
}

impl RouteMatcher {
    fn matches(&self, method: &Method, path: &str) -> bool {
        if self.exclude.iter().any(|route| route.matches(method, path)) {
            return false;
        }

        if self.include.is_empty() {
            return true;
        }

        self.include.iter().any(|route| route.matches(method, path))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiddlewareRoute {
    path: String,
    methods: Option<Vec<Method>>,
}

impl MiddlewareRoute {
    pub fn path(path: impl Into<String>) -> Self {
        Self {
            path: normalize_path(path.into()),
            methods: None,
        }
    }

    pub fn methods<I>(path: impl Into<String>, methods: I) -> Self
    where
        I: IntoIterator<Item = Method>,
    {
        Self {
            path: normalize_path(path.into()),
            methods: Some(methods.into_iter().collect()),
        }
    }

    pub fn get(path: impl Into<String>) -> Self {
        Self::methods(path, [Method::GET])
    }

    pub fn post(path: impl Into<String>) -> Self {
        Self::methods(path, [Method::POST])
    }

    pub fn put(path: impl Into<String>) -> Self {
        Self::methods(path, [Method::PUT])
    }

    pub fn delete(path: impl Into<String>) -> Self {
        Self::methods(path, [Method::DELETE])
    }

    fn matches(&self, method: &Method, path: &str) -> bool {
        if !path_matches_prefix(path, &self.path) {
            return false;
        }

        match &self.methods {
            Some(methods) => methods.iter().any(|candidate| candidate == method),
            None => true,
        }
    }
}

impl From<&str> for MiddlewareRoute {
    fn from(value: &str) -> Self {
        Self::path(value)
    }
}

impl From<String> for MiddlewareRoute {
    fn from(value: String) -> Self {
        Self::path(value)
    }
}

#[derive(Default)]
pub struct MiddlewareConsumer {
    bindings: Vec<MiddlewareBinding>,
}

impl MiddlewareConsumer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply<T>(&mut self) -> MiddlewareBindingBuilder<'_>
    where
        T: NestMiddleware + Default,
    {
        MiddlewareBindingBuilder::new(self, Arc::new(T::default()))
    }

    pub fn apply_instance<T>(&mut self, middleware: T) -> MiddlewareBindingBuilder<'_>
    where
        T: NestMiddleware,
    {
        MiddlewareBindingBuilder::new(self, Arc::new(middleware))
    }

    pub fn into_bindings(self) -> Vec<MiddlewareBinding> {
        self.bindings
    }
}

pub struct MiddlewareBindingBuilder<'a> {
    consumer: &'a mut MiddlewareConsumer,
    middleware: Arc<dyn NestMiddleware>,
    exclude: Vec<MiddlewareRoute>,
}

impl<'a> MiddlewareBindingBuilder<'a> {
    fn new(consumer: &'a mut MiddlewareConsumer, middleware: Arc<dyn NestMiddleware>) -> Self {
        Self {
            consumer,
            middleware,
            exclude: Vec::new(),
        }
    }

    pub fn exclude<I, S>(mut self, routes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<MiddlewareRoute>,
    {
        self.exclude = routes.into_iter().map(Into::into).collect();
        self
    }

    pub fn for_all_routes(self) -> &'a mut MiddlewareConsumer {
        self.register(Vec::new())
    }

    pub fn for_routes<I, S>(self, routes: I) -> &'a mut MiddlewareConsumer
    where
        I: IntoIterator<Item = S>,
        S: Into<MiddlewareRoute>,
    {
        let include = routes.into_iter().map(Into::into).collect();
        self.register(include)
    }

    fn register(self, include: Vec<MiddlewareRoute>) -> &'a mut MiddlewareConsumer {
        framework_log_event(
            "middleware_register",
            &[
                ("include", format!("{include:?}")),
                ("exclude", format!("{:?}", self.exclude)),
            ],
        );
        self.consumer.bindings.push(MiddlewareBinding {
            middleware: self.middleware,
            matcher: RouteMatcher {
                include,
                exclude: self.exclude,
            },
        });
        self.consumer
    }
}

pub fn run_middleware_chain(
    middlewares: Arc<Vec<MiddlewareBinding>>,
    index: usize,
    req: Request<Body>,
    terminal: NextFn,
) -> NextFuture {
    if index >= middlewares.len() {
        return terminal(req);
    }

    let binding = middlewares[index].clone();
    if !binding.matches(req.method(), req.uri().path()) {
        return run_middleware_chain(middlewares, index + 1, req, terminal);
    }

    let middlewares_for_next = Arc::clone(&middlewares);
    let terminal_for_next = Arc::clone(&terminal);
    let next_fn: NextFn = Arc::new(move |next_req| {
        run_middleware_chain(
            Arc::clone(&middlewares_for_next),
            index + 1,
            next_req,
            Arc::clone(&terminal_for_next),
        )
    });

    binding.middleware.handle(req, next_fn)
}

fn normalize_path(path: String) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }

    if trimmed.starts_with('/') {
        trimmed.trim_end_matches('/').to_string()
    } else {
        format!("/{}", trimmed.trim_end_matches('/'))
    }
}

fn path_matches_prefix(path: &str, prefix: &str) -> bool {
    if prefix == "/" {
        return true;
    }

    path == prefix || path.starts_with(&format!("{prefix}/"))
}

#[cfg(test)]
mod tests {
    use axum::http::Method;

    use super::{normalize_path, path_matches_prefix, MiddlewareRoute, RouteMatcher};

    #[test]
    fn matcher_supports_prefix_matching_and_excludes() {
        let matcher = RouteMatcher {
            include: vec![MiddlewareRoute::path("/api")],
            exclude: vec![MiddlewareRoute::path("/api/health")],
        };

        assert!(matcher.matches(&Method::GET, "/api/users"));
        assert!(!matcher.matches(&Method::GET, "/api/health"));
        assert!(!matcher.matches(&Method::GET, "/admin"));
    }

    #[test]
    fn normalize_path_handles_empty_and_trailing_slashes() {
        assert_eq!(normalize_path("".to_string()), "/");
        assert_eq!(normalize_path("/users/".to_string()), "/users");
        assert_eq!(normalize_path("users".to_string()), "/users");
    }

    #[test]
    fn prefix_matching_requires_boundary() {
        assert!(path_matches_prefix("/users/1", "/users"));
        assert!(path_matches_prefix("/users", "/users"));
        assert!(!path_matches_prefix("/users-list", "/users"));
    }

    #[test]
    fn matcher_can_target_specific_http_methods() {
        let matcher = RouteMatcher {
            include: vec![MiddlewareRoute::get("/admin")],
            exclude: Vec::new(),
        };

        assert!(matcher.matches(&Method::GET, "/admin/users"));
        assert!(!matcher.matches(&Method::POST, "/admin/users"));
    }
}
