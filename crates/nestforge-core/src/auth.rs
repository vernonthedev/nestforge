use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{request::request_id_from_extensions, HttpException};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthIdentity {
    pub subject: String,
    pub roles: Vec<String>,
    #[serde(default)]
    pub claims: BTreeMap<String, Value>,
}

impl AuthIdentity {
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            roles: Vec::new(),
            claims: BTreeMap::new(),
        }
    }

    pub fn with_roles<I, S>(mut self, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_claim(mut self, key: impl Into<String>, value: Value) -> Self {
        self.claims.insert(key.into(), value);
        self
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|candidate| candidate == role)
    }

    pub fn require_role(&self, role: &str) -> Result<(), HttpException> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(HttpException::forbidden(format!(
                "Missing required role `{role}`"
            )))
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub Arc<AuthIdentity>);

impl AuthUser {
    pub fn into_inner(self) -> Arc<AuthIdentity> {
        self.0
    }

    pub fn value(&self) -> &AuthIdentity {
        &self.0
    }
}

impl Deref for AuthUser {
    type Target = AuthIdentity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_extensions(&parts.extensions);
        let identity = parts
            .extensions
            .get::<Arc<AuthIdentity>>()
            .cloned()
            .ok_or_else(|| {
                HttpException::unauthorized("Authentication required")
                    .with_optional_request_id(request_id)
            })?;

        Ok(Self(identity))
    }
}

#[derive(Debug, Clone)]
pub struct BearerToken(pub Arc<str>);

impl BearerToken {
    pub fn value(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> String {
        self.0.as_ref().to_string()
    }
}

impl Deref for BearerToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<S> FromRequestParts<S> for BearerToken
where
    S: Send + Sync,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request_id_from_extensions(&parts.extensions);
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                HttpException::unauthorized("Missing Authorization header")
                    .with_optional_request_id(request_id.clone())
            })?;

        let token = header
            .strip_prefix("Bearer ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                HttpException::unauthorized("Invalid bearer token")
                    .with_optional_request_id(request_id)
            })?;

        Ok(Self(Arc::<str>::from(token.to_string())))
    }
}
