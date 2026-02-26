use std::{future::Future, pin::Pin, sync::Arc};

use axum::{
    body::Body,
    extract::Request,
    http::{Method, Uri},
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

pub fn run_guards(
    guards: &[Arc<dyn Guard>],
    ctx: &RequestContext,
) -> Result<(), HttpException> {
    for guard in guards {
        guard.can_activate(ctx)?;
    }
    Ok(())
}
