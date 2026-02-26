use std::ops::Deref;

use axum::{
    extract::{FromRequest, FromRequestParts, Path},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

use crate::{HttpException, Validate};

/*
Param<T> = path param wrapper

User writes:
id: Param<u64>

Instead of:
Path(id): Path<u64>
*/
pub struct Param<T>(pub T);

impl<T> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|_| HttpException::bad_request("Invalid route parameter"))?;

        Ok(Self(value))
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
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|_| HttpException::bad_request("Invalid JSON body"))?;

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
}

impl<S, T> FromRequest<S> for ValidatedBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send + 'static,
{
    type Rejection = HttpException;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|_| HttpException::bad_request("Invalid JSON body"))?;

        value
            .validate()
            .map_err(HttpException::bad_request_validation)?;

        Ok(Self(value))
    }
}
