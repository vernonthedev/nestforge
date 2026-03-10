use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use axum::{
    body::Body,
    extract::Request,
    http::{request::Parts, Method, Uri},
    middleware::Next,
    response::IntoResponse,
    response::Response,
};

use crate::AuthIdentity;
use crate::HttpException;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub method: Method,
    pub uri: Uri,
    pub request_id: Option<String>,
    pub auth_identity: Option<Arc<AuthIdentity>>,
}

impl RequestContext {
    pub fn from_parts(parts: &Parts) -> Self {
        Self {
            method: parts.method.clone(),
            uri: parts.uri.clone(),
            request_id: crate::request::request_id_from_extensions(&parts.extensions),
            auth_identity: parts.extensions.get::<Arc<AuthIdentity>>().cloned(),
        }
    }

    pub fn from_request(req: &Request) -> Self {
        Self {
            method: req.method().clone(),
            uri: req.uri().clone(),
            request_id: crate::request::request_id_from_extensions(req.extensions()),
            auth_identity: req.extensions().get::<Arc<AuthIdentity>>().cloned(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_identity.is_some()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.auth_identity
            .as_ref()
            .map(|identity| identity.has_role(role))
            .unwrap_or(false)
    }
}

pub trait ExceptionFilter: Send + Sync + 'static {
    fn catch(&self, exception: HttpException, ctx: &RequestContext) -> HttpException;
}

pub trait Guard: Send + Sync + 'static {
    fn can_activate(&self, ctx: &RequestContext) -> Result<(), HttpException>;
}

#[derive(Default)]
pub struct RequireAuthenticationGuard;

impl Guard for RequireAuthenticationGuard {
    fn can_activate(&self, ctx: &RequestContext) -> Result<(), HttpException> {
        if ctx.is_authenticated() {
            Ok(())
        } else {
            Err(HttpException::unauthorized("Authentication required"))
        }
    }
}

pub struct RoleRequirementsGuard {
    roles: Vec<String>,
}

impl RoleRequirementsGuard {
    pub fn new<I, S>(roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }
}

impl Guard for RoleRequirementsGuard {
    fn can_activate(&self, ctx: &RequestContext) -> Result<(), HttpException> {
        if !ctx.is_authenticated() {
            return Err(HttpException::unauthorized("Authentication required"));
        }

        if self.roles.iter().any(|role| ctx.has_role(role)) {
            Ok(())
        } else {
            Err(HttpException::forbidden(format!(
                "Missing required role. Expected one of: {}",
                self.roles.join(", ")
            )))
        }
    }
}

pub type NextFuture = Pin<Box<dyn Future<Output = Response> + Send>>;
pub type NextFn = Arc<dyn Fn(Request<Body>) -> NextFuture + Send + Sync + 'static>;

pub trait Interceptor: Send + Sync + 'static {
    fn around(&self, ctx: RequestContext, req: Request<Body>, next: NextFn) -> NextFuture;
}

pub fn run_guards(guards: &[Arc<dyn Guard>], ctx: &RequestContext) -> Result<(), HttpException> {
    for guard in guards {
        guard.can_activate(ctx)?;
    }
    Ok(())
}

fn next_to_fn(next: Next) -> NextFn {
    let next = Arc::new(Mutex::new(Some(next)));

    Arc::new(move |req: Request<Body>| {
        let next = Arc::clone(&next);
        Box::pin(async move {
            let next = {
                let mut guard = match next.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return HttpException::internal_server_error("Pipeline lock poisoned")
                            .into_response();
                    }
                };
                guard.take()
            };

            match next {
                Some(next) => next.run(req).await,
                std::option::Option::None => crate::HttpException::internal_server_error(
                    "Pipeline next called multiple times",
                )
                .into_response(),
            }
        })
    })
}

fn run_interceptor_chain(
    interceptors: Arc<Vec<Arc<dyn Interceptor>>>,
    index: usize,
    ctx: RequestContext,
    req: Request<Body>,
    terminal: NextFn,
) -> NextFuture {
    if index >= interceptors.len() {
        return terminal(req);
    }

    let current = Arc::clone(&interceptors[index]);
    let interceptors_for_next = Arc::clone(&interceptors);
    let ctx_for_next = ctx.clone();
    let terminal_for_next = Arc::clone(&terminal);

    let next_fn: NextFn = Arc::new(move |next_req: Request<Body>| {
        run_interceptor_chain(
            Arc::clone(&interceptors_for_next),
            index + 1,
            ctx_for_next.clone(),
            next_req,
            Arc::clone(&terminal_for_next),
        )
    });

    current.around(ctx, req, next_fn)
}

pub async fn execute_pipeline(
    req: Request<Body>,
    next: Next,
    guards: Arc<Vec<Arc<dyn Guard>>>,
    interceptors: Arc<Vec<Arc<dyn Interceptor>>>,
    filters: Arc<Vec<Arc<dyn ExceptionFilter>>>,
) -> Response {
    let ctx = RequestContext::from_request(&req);

    if let Err(err) = run_guards(guards.as_slice(), &ctx) {
        return apply_exception_filters(err, &ctx, filters.as_slice()).into_response();
    }

    let terminal = next_to_fn(next);
    run_interceptor_chain(interceptors, 0, ctx, req, terminal).await
}

pub fn apply_exception_filters(
    mut exception: HttpException,
    ctx: &RequestContext,
    filters: &[Arc<dyn ExceptionFilter>],
) -> HttpException {
    for filter in filters {
        exception = filter.catch(exception, ctx);
    }

    exception
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::Method;

    use crate::{AuthIdentity, Guard};

    use super::{RequestContext, RequireAuthenticationGuard, RoleRequirementsGuard};

    fn anonymous_context() -> RequestContext {
        RequestContext {
            method: Method::GET,
            uri: "/".parse().expect("uri should parse"),
            request_id: None,
            auth_identity: None,
        }
    }

    fn authenticated_context(roles: &[&str]) -> RequestContext {
        RequestContext {
            method: Method::GET,
            uri: "/".parse().expect("uri should parse"),
            request_id: None,
            auth_identity: Some(Arc::new(
                AuthIdentity::new("user-1").with_roles(roles.iter().copied()),
            )),
        }
    }

    #[test]
    fn authentication_guard_rejects_anonymous_requests() {
        let guard = RequireAuthenticationGuard;

        assert!(guard.can_activate(&anonymous_context()).is_err());
        assert!(guard.can_activate(&authenticated_context(&[])).is_ok());
    }

    #[test]
    fn role_guard_accepts_any_matching_role() {
        let guard = RoleRequirementsGuard::new(["admin", "support"]);

        assert!(guard
            .can_activate(&authenticated_context(&["support"]))
            .is_ok());
        assert!(guard
            .can_activate(&authenticated_context(&["viewer"]))
            .is_err());
        assert!(guard.can_activate(&anonymous_context()).is_err());
    }
}
