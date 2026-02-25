use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
};

use crate::{Container, HttpException};

/*
Inject<T> = DI extractor wrapper

Now users can write:
users: Inject<UsersService>

and NestForge resolves it from the shared DI container automatically.
*/
pub struct Inject<T>(Arc<T>);

impl<T> Inject<T>
where
    T: Send + Sync + 'static,
{
    /*
    Manual resolve helper (still useful internally/tests)
    */
    pub fn from(container: &Container) -> Result<Self> {
        let inner = container.resolve::<T>()?;
        Ok(Self(inner))
    }

    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T> Deref for Inject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/*
Magic extractor:
Axum gives us Container state -> we resolve T from DI -> handler gets Inject<T>
*/
impl<T> FromRequestParts<Container> for Inject<T>
where
    T: Send + Sync + 'static,
{
    type Rejection = HttpException;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Container,
    ) -> Result<Self, Self::Rejection> {
        let State(container): State<Container> = State::from_request_parts(parts, state)
            .await
            .map_err(|_| HttpException::internal_server_error("Container state not available"))?;

        Self::from(&container)
            .map_err(|_| HttpException::internal_server_error("Dependency not registered"))
    }
}