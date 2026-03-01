use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteResponseDocumentation {
    pub status: u16,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteDocumentation {
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub responses: Vec<RouteResponseDocumentation>,
    pub requires_auth: bool,
    pub required_roles: Vec<String>,
}

impl RouteDocumentation {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            summary: None,
            description: None,
            tags: Vec::new(),
            responses: vec![RouteResponseDocumentation {
                status: 200,
                description: "OK".to_string(),
            }],
            requires_auth: false,
            required_roles: Vec::new(),
        }
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_responses(mut self, responses: Vec<RouteResponseDocumentation>) -> Self {
        self.responses = responses;
        self
    }

    pub fn requires_auth(mut self) -> Self {
        self.requires_auth = true;
        self
    }

    pub fn with_required_roles<I, S>(mut self, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_roles = roles.into_iter().map(Into::into).collect();
        self
    }
}

pub trait DocumentedController: Send + Sync + 'static {
    fn route_docs() -> Vec<RouteDocumentation> {
        Vec::new()
    }
}
