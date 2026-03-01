use std::{ops::Deref, sync::Arc};

use axum::{
    extract::{FromRequest, FromRequestParts, Path, Query as AxumQuery},
    http::{request::Parts, Extensions, HeaderMap},
};
use serde::de::DeserializeOwned;

use crate::{HttpException, Validate};

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
    extensions.get::<RequestId>().map(|request_id| request_id.value().to_string())
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

        value
            .validate()
            .map_err(|errors| {
                HttpException::bad_request_validation(errors)
                    .with_optional_request_id(request_id)
            })?;

        Ok(Self(value))
    }
}
