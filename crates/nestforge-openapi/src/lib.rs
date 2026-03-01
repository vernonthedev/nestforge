use axum::{response::Html, routing::get, Json, Router};
use nestforge_core::RouteDocumentation;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiRoute {
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub requires_auth: bool,
    pub responses: Vec<OpenApiResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiResponse {
    pub status: u16,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiDoc {
    pub title: String,
    pub version: String,
    pub routes: Vec<OpenApiRoute>,
}

impl OpenApiDoc {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            routes: Vec::new(),
        }
    }

    pub fn add_route(mut self, method: impl Into<String>, path: impl Into<String>) -> Self {
        self.routes.push(OpenApiRoute {
            method: method.into(),
            path: path.into(),
            summary: None,
            description: None,
            tags: Vec::new(),
            requires_auth: false,
            responses: vec![OpenApiResponse {
                status: 200,
                description: "OK".to_string(),
            }],
        });
        self
    }

    pub fn from_routes(
        title: impl Into<String>,
        version: impl Into<String>,
        routes: Vec<RouteDocumentation>,
    ) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            routes: routes
                .into_iter()
                .map(|route| OpenApiRoute {
                    method: route.method,
                    path: route.path,
                    summary: route.summary,
                    description: route.description,
                    tags: route.tags,
                    requires_auth: route.requires_auth,
                    responses: route
                        .responses
                        .into_iter()
                        .map(|response| OpenApiResponse {
                            status: response.status,
                            description: response.description,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    pub fn to_openapi_json(&self) -> Value {
        let mut paths = serde_json::Map::new();
        for route in &self.routes {
            let method = route.method.to_lowercase();
            let entry = paths.entry(route.path.clone()).or_insert_with(|| json!({}));
            let obj = entry.as_object_mut().expect("path entry object");
            let responses = route
                .responses
                .iter()
                .map(|response| {
                    (
                        response.status.to_string(),
                        json!({ "description": response.description }),
                    )
                })
                .collect::<serde_json::Map<String, Value>>();
            obj.insert(
                method,
                json!({
                    "summary": route.summary,
                    "description": route.description,
                    "tags": route.tags,
                    "responses": responses,
                    "security": if route.requires_auth { json!([{"bearerAuth": []}]) } else { json!([]) }
                }),
            );
        }

        json!({
            "openapi": "3.1.0",
            "info": {
                "title": self.title,
                "version": self.version
            },
            "components": {
                "securitySchemes": {
                    "bearerAuth": {
                        "type": "http",
                        "scheme": "bearer",
                        "bearerFormat": "JWT"
                    }
                }
            },
            "paths": paths
        })
    }
}

pub fn docs_router(doc: OpenApiDoc) -> Router {
    let openapi = doc.to_openapi_json();
    let routes_html = doc
        .routes
        .iter()
        .map(|route| {
            let summary = route.summary.clone().unwrap_or_else(|| "No summary".to_string());
            format!(
                "<li><strong>{}</strong> <code>{}</code> - {}</li>",
                route.method, route.path, summary
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let docs_html = format!(
        r#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>NestForge API Docs</title></head>
<body>
  <h1>NestForge API Docs</h1>
  <p>OpenAPI JSON is available at <code>/openapi.json</code>.</p>
  <ul>{routes_html}</ul>
</body>
</html>"#
    );

    Router::new()
        .route(
            "/openapi.json",
            get({
                let payload = openapi.clone();
                move || async move { Json(payload.clone()) }
            }),
        )
        .route("/docs", get(move || async move { Html(docs_html.clone()) }))
}
