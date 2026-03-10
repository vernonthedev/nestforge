use std::{ops::Deref, sync::Arc};

use axum::{
    extract::{FromRequest, FromRequestParts, Path, Query as AxumQuery},
    http::{request::Parts, Extensions, HeaderMap},
};
use serde::de::DeserializeOwned;

use crate::{HttpException, RequestContext, Validate};

pub trait Pipe<Input>: Send + Sync + 'static {
    type Output;

    fn transform(value: Input, ctx: &RequestContext) -> Result<Self::Output, HttpException>;
}

pub trait RequestDecorator: Send + Sync + 'static {
    type Output: Send + 'static;

    fn extract(ctx: &RequestContext, parts: &Parts) -> Result<Self::Output, HttpException>;
}

#[derive(Debug, Clone)]
pub struct RequestId(pub Arc<str>);

impl RequestId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Arc::<str>::from(value.into()))
    }

    pub fn into_inner(self) -> String {
        self.0.as_ref().to_string()
    }

    pub fn value(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for RequestId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub fn request_id_from_extensions(extensions: &Extensions) -> Option<String> {
    extensions
        .get::<RequestId>()
        .map(|request_id| request_id.value().to_string())
}

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .ok_or_else(|| HttpException::internal_server_error("Request id not available"))
    }
}

/*
Param<T> = path param wrapper

User writes:
id: Param<u64>

Instead of:
Path(id): Path<u64>
*/
#[derive(Debug, Clone, Copy)]
pub struct Param<T>(pub T);

impl<T> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Decorated<T>
where
    T: RequestDecorator,
{
    value: T::Output,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Deref for Decorated<T>
where
    T: RequestDecorator,
{
    type Target = T::Output;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Decorated<T>
where
    T: RequestDecorator,
{
    pub fn into_inner(self) -> T::Output {
        self.value
    }
}

impl<S, T> FromRequestParts<S> for Decorated<T>
where
    S: Send + Sync,
    T: RequestDecorator,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = RequestContext::from_parts(parts);
        let value = T::extract(&ctx, parts)?;

        Ok(Self {
            value,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T> Param<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn value(self) -> T {
        self.0
    }
}

/*
Extract Param<T> from route path params.
*/
impl<S, T> FromRequestParts<S> for Param<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_extensions(&parts.extensions);
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid route parameter")
                    .with_optional_request_id(request_id)
            })?;

        Ok(Self(value))
    }
}

pub struct PipedParam<T, P>
where
    P: Pipe<T>,
{
    value: P::Output,
    _marker: std::marker::PhantomData<(T, P)>,
}

impl<T, P> Deref for PipedParam<T, P>
where
    P: Pipe<T>,
{
    type Target = P::Output;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, P> PipedParam<T, P>
where
    P: Pipe<T>,
{
    pub fn into_inner(self) -> P::Output {
        self.value
    }
}

impl<S, T, P> FromRequestParts<S> for PipedParam<T, P>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
    P: Pipe<T>,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid route parameter")
                    .with_optional_request_id(request_id_from_extensions(&parts.extensions))
            })?;
        let ctx = RequestContext::from_parts(parts);
        let value = P::transform(value, &ctx)?;

        Ok(Self {
            value,
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Query<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn value(self) -> T {
        self.0
    }
}

impl<S, T> FromRequestParts<S> for Query<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_extensions(&parts.extensions);
        let AxumQuery(value) = AxumQuery::<T>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid query parameters")
                    .with_optional_request_id(request_id)
            })?;

        Ok(Self(value))
    }
}

pub struct PipedQuery<T, P>
where
    P: Pipe<T>,
{
    value: P::Output,
    _marker: std::marker::PhantomData<(T, P)>,
}

impl<T, P> Deref for PipedQuery<T, P>
where
    P: Pipe<T>,
{
    type Target = P::Output;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, P> PipedQuery<T, P>
where
    P: Pipe<T>,
{
    pub fn into_inner(self) -> P::Output {
        self.value
    }
}

impl<S, T, P> FromRequestParts<S> for PipedQuery<T, P>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
    P: Pipe<T>,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AxumQuery(value) = AxumQuery::<T>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid query parameters")
                    .with_optional_request_id(request_id_from_extensions(&parts.extensions))
            })?;
        let ctx = RequestContext::from_parts(parts);
        let value = P::transform(value, &ctx)?;

        Ok(Self {
            value,
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Headers(pub HeaderMap);

impl Headers {
    pub fn get(&self, name: &str) -> Option<&axum::http::HeaderValue> {
        self.0.get(name)
    }
}

impl Deref for Headers {
    type Target = HeaderMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for Headers
where
    S: Send + Sync,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(parts.headers.clone()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cookies {
    values: std::collections::BTreeMap<String, String>,
}

impl Cookies {
    pub fn new<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            values: pairs
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
}

impl<S> FromRequestParts<S> for Cookies
where
    S: Send + Sync,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let mut cookies = std::collections::BTreeMap::new();
        if let Some(header) = parts.headers.get(axum::http::header::COOKIE) {
            if let Ok(raw) = header.to_str() {
                for pair in raw.split(';') {
                    let trimmed = pair.trim();
                    if let Some((name, value)) = trimmed.split_once('=') {
                        cookies.insert(name.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }

        Ok(Self { values: cookies })
    }
}

/*
Body<T> = JSON request body wrapper

User writes:
body: Body<CreateUserDto>

Instead of:
Json(dto): Json<CreateUserDto>
*/
pub struct Body<T>(pub T);

impl<T> Deref for Body<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Body<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn value(self) -> T {
        self.0
    }
}

/*
Extract Body<T> from JSON request body.
*/
impl<S, T> FromRequest<S> for Body<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = HttpException;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_extensions(req.extensions());
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid JSON body").with_optional_request_id(request_id)
            })?;

        Ok(Self(value))
    }
}

pub struct PipedBody<T, P>
where
    P: Pipe<T>,
{
    value: P::Output,
    _marker: std::marker::PhantomData<(T, P)>,
}

impl<T, P> Deref for PipedBody<T, P>
where
    P: Pipe<T>,
{
    type Target = P::Output;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, P> PipedBody<T, P>
where
    P: Pipe<T>,
{
    pub fn into_inner(self) -> P::Output {
        self.value
    }
}

impl<S, T, P> FromRequest<S> for PipedBody<T, P>
where
    S: Send + Sync,
    T: DeserializeOwned + Send + 'static,
    P: Pipe<T>,
{
    type Rejection = HttpException;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = RequestContext::from_request(&req);
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid JSON body")
                    .with_optional_request_id(ctx.request_id.clone())
            })?;
        let value = P::transform(value, &ctx)?;

        Ok(Self {
            value,
            _marker: std::marker::PhantomData,
        })
    }
}

pub struct ValidatedBody<T>(pub T);

impl<T> Deref for ValidatedBody<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ValidatedBody<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn value(self) -> T {
        self.0
    }
}

impl<S, T> FromRequest<S> for ValidatedBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send + 'static,
{
    type Rejection = HttpException;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_extensions(req.extensions());
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|_| {
                HttpException::bad_request("Invalid JSON body")
                    .with_optional_request_id(request_id.clone())
            })?;

        value.validate().map_err(|errors| {
            HttpException::bad_request_validation(errors).with_optional_request_id(request_id)
        })?;

        Ok(Self(value))
    }
}
