use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{request::request_id_from_extensions, HttpException};

/**
 * Authentication Identity
 *
 * Represents an authenticated user's identity within the NestForge framework.
 * Contains the subject (typically user ID), roles, and additional custom claims.
 *
 * # Fields
 * - `subject`: The unique identifier for the user (e.g., user ID, email)
 * - `roles`: List of role names the user possesses
 * - `claims`: Additional key-value pairs for custom authentication data
 *
 * # Usage
 * This type is automatically populated by the framework's authentication
 * middleware and can be accessed in handlers via the `AuthUser` extractor.
 */
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthIdentity {
    pub subject: String,
    pub roles: Vec<String>,
    #[serde(default)]
    pub claims: BTreeMap<String, Value>,
}

impl AuthIdentity {
    /**
     * Creates a new authentication identity.
     *
     * # Arguments
     * - `subject`: The unique identifier for the user
     *
     * # Example
     * ```rust
     * let identity = AuthIdentity::new("user-123");
     * ```
     */
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            roles: Vec::new(),
            claims: BTreeMap::new(),
        }
    }

    /**
     * Adds roles to the authentication identity.
     *
     * # Arguments
     * - `roles`: An iterator of role names to assign
     */
    pub fn with_roles<I, S>(mut self, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    /**
     * Adds a custom claim to the authentication identity.
     *
     * # Arguments
     * - `key`: The claim key
     * - `value`: The claim value
     */
    pub fn with_claim(mut self, key: impl Into<String>, value: Value) -> Self {
        self.claims.insert(key.into(), value);
        self
    }

    /**
     * Checks if the identity has a specific role.
     *
     * # Arguments
     * - `role`: The role name to check
     *
     * Returns true if the role is present in the identity's roles.
     */
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|candidate| candidate == role)
    }

    /**
     * Requires a specific role, returning an error if not present.
     *
     * # Arguments
     * - `role`: The required role name
     *
     * Returns Ok if the role is present, or a forbidden HttpException if not.
     */
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

/**
 * AuthUser Extractor
 *
 * A request extractor that provides mandatory authentication.
 * Fails with 401 Unauthorized if no authenticated identity is present.
 *
 * # Usage
 * ```rust
 * async fn handler(user: AuthUser) -> impl IntoResponse {
 *     format!("Hello, {}", user.subject)
 * }
 * ```
 */
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

/**
 * OptionalAuthUser Extractor
 *
 * A request extractor that provides optional authentication.
 * Unlike `AuthUser`, this succeeds even when no identity is present,
 * returning None in that case.
 *
 * # Usage
 * ```rust
 * async fn handler(user: OptionalAuthUser) -> impl IntoResponse {
 *     match user.value() {
 *         Some(identity) => format!("Hello, {}", identity.subject),
 *         None => "Hello, guest".to_string(),
 *     }
 * }
 * ```
 */
#[derive(Debug, Clone, Default)]
pub struct OptionalAuthUser(pub Option<Arc<AuthIdentity>>);

/**
 * BearerToken Extractor
 *
 * A request extractor that extracts the bearer token from the
 * Authorization header. Useful for custom authentication schemes.
 *
 * # Response
 * Returns the token string without the "Bearer " prefix.
 *
 * # Errors
 * - 401 if Authorization header is missing
 * - 401 if header doesn't start with "Bearer "
 * - 401 if token is empty after trimming
 */
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
