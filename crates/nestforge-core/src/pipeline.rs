use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    http::{Method, Uri},
    response::IntoResponse,
    response::Response,
};

use crate::HttpException;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub method: Method,
    pub uri: Uri,
}

impl RequestContext {
    pub fn from_request(req: &Request) -> Self {
        Self {
            method: req.method().clone(),
            uri: req.uri().clone(),
        }
    }
}

pub trait Guard: Send + Sync + 'static {
    fn can_activate(&self, ctx: &RequestContext) -> Result<(), HttpException>;
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
                let mut guard = next.lock().expect("next lock poisoned");
                guard.take()
            };

            match next {
                Some(next) => next.run(req).await,
                None => HttpException::internal_server_error("Pipeline next called multiple times")
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
) -> Response {
    let ctx = RequestContext::from_request(&req);

    if let Err(err) = run_guards(guards.as_slice(), &ctx) {
        return err.into_response();
    }

    let terminal = next_to_fn(next);
    run_interceptor_chain(interceptors, 0, ctx, req, terminal).await
}
