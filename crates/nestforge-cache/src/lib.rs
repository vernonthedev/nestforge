use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{header::CONTENT_TYPE, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use nestforge_core::{framework_log_event, Container, Interceptor, NextFn, NextFuture, RequestContext};
use nestforge_data::CacheStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedHttpResponse {
    status: u16,
    body: String,
    content_type: Option<String>,
}

impl CachedHttpResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK);
        let mut response = (status, self.body).into_response();

        if let Some(content_type) = self.content_type {
            if let Ok(header) = HeaderValue::from_str(&content_type) {
                response.headers_mut().insert(CONTENT_TYPE, header);
            }
        }

        response
    }
}

pub trait CachePolicy: Default + Clone + Send + Sync + 'static {
    type Store: CacheStore + Send + Sync + 'static;

    fn cache_key(&self, ctx: &RequestContext, req: &Request<Body>) -> Option<String> {
        if ctx.method != Method::GET {
            return None;
        }

        Some(format!(
            "{}:{}",
            std::any::type_name::<Self>(),
            req.uri()
        ))
    }

    fn ttl_seconds(&self) -> Option<u64> {
        None
    }

    fn should_cache_response(&self, response: &Response) -> bool {
        response.status() == StatusCode::OK
    }
}

#[derive(Debug, Clone)]
pub struct CacheInterceptor<P>
where
    P: CachePolicy,
{
    policy: P,
}

impl<P> Default for CacheInterceptor<P>
where
    P: CachePolicy,
{
    fn default() -> Self {
        Self {
            policy: P::default(),
        }
    }
}

impl<P> CacheInterceptor<P>
where
    P: CachePolicy,
{
    pub fn new(policy: P) -> Self {
        Self { policy }
    }
}

impl<P> Interceptor for CacheInterceptor<P>
where
    P: CachePolicy,
{
    fn around(&self, ctx: RequestContext, req: Request<Body>, next: NextFn) -> NextFuture {
        let policy = self.policy.clone();

        Box::pin(async move {
            let Some(container) = req.extensions().get::<Container>().cloned() else {
                return (next)(req).await;
            };

            let Some(cache_key) = policy.cache_key(&ctx, &req) else {
                return (next)(req).await;
            };

            let Ok(store) = container.resolve::<P::Store>() else {
                return (next)(req).await;
            };

            if let Ok(Some(cached)) = store.get(&cache_key).await {
                match serde_json::from_str::<CachedHttpResponse>(&cached) {
                    Ok(response) => {
                        framework_log_event(
                            "response_cache_hit",
                            &[("key", cache_key.clone())],
                        );
                        return response.into_response();
                    }
                    Err(err) => {
                        framework_log_event(
                            "response_cache_deserialize_failed",
                            &[
                                ("key", cache_key.clone()),
                                ("error", err.to_string()),
                            ],
                        );
                    }
                }
            }

            let response = (next)(req).await;
            if !policy.should_cache_response(&response) {
                return response;
            }

            let (parts, body) = response.into_parts();
            let bytes = match to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(err) => {
                    framework_log_event(
                        "response_cache_read_failed",
                        &[
                            ("key", cache_key),
                            ("error", err.to_string()),
                        ],
                    );
                    return nestforge_core::HttpException::internal_server_error(
                        "Failed to read response body for caching",
                    )
                    .into_response();
                }
            };

            let content_type = parts
                .headers
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);

            let response_for_client = Response::from_parts(parts, Body::from(bytes.clone()));

            let Ok(body) = String::from_utf8(bytes.to_vec()) else {
                return response_for_client;
            };

            let payload = CachedHttpResponse {
                status: response_for_client.status().as_u16(),
                body,
                content_type,
            };

            match serde_json::to_string(&payload) {
                Ok(serialized) => {
                    if let Err(err) = store
                        .set(&cache_key, &serialized, policy.ttl_seconds())
                        .await
                    {
                        framework_log_event(
                            "response_cache_store_failed",
                            &[
                                ("key", cache_key),
                                ("error", err.to_string()),
                            ],
                        );
                    }
                }
                Err(err) => {
                    framework_log_event(
                        "response_cache_serialize_failed",
                        &[
                            ("key", cache_key),
                            ("error", err.to_string()),
                        ],
                    );
                }
            }

            response_for_client
        })
    }
}

#[derive(Debug)]
pub struct DefaultCachePolicy<S>
where
    S: CacheStore + Send + Sync + 'static,
{
    _marker: std::marker::PhantomData<fn() -> S>,
}

impl<S> Clone for DefaultCachePolicy<S>
where
    S: CacheStore + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl<S> Default for DefaultCachePolicy<S>
where
    S: CacheStore + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S> CachePolicy for DefaultCachePolicy<S>
where
    S: CacheStore + Send + Sync + 'static,
{
    type Store = S;
}

pub fn cached_response_interceptor<S>() -> CacheInterceptor<DefaultCachePolicy<S>>
where
    S: CacheStore + Send + Sync + 'static,
{
    CacheInterceptor::default()
}
