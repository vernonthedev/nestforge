use axum::{routing::get, Json, Router};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiRoute {
    pub method: String,
    pub path: String,
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
        });
        self
    }

    pub fn to_openapi_json(&self) -> Value {
        let mut paths = serde_json::Map::new();
        for route in &self.routes {
            let method = route.method.to_lowercase();
            let entry = paths.entry(route.path.clone()).or_insert_with(|| json!({}));
            let obj = entry.as_object_mut().expect("path entry object");
            obj.insert(
                method,
                json!({
                    "responses": {
                        "200": { "description": "OK" }
                    }
                }),
            );
        }

        json!({
            "openapi": "3.1.0",
            "info": {
                "title": self.title,
                "version": self.version
            },
            "paths": paths
        })
    }
}

pub fn docs_router(doc: OpenApiDoc) -> Router {
    let openapi = doc.to_openapi_json();
    let docs_html = r#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>NestForge API Docs</title></head>
<body>
  <h1>NestForge API Docs</h1>
  <p>OpenAPI JSON is available at <code>/openapi.json</code>.</p>
</body>
</html>"#;

    Router::new()
        .route(
            "/openapi.json",
            get({
                let payload = openapi.clone();
                move || async move { Json(payload.clone()) }
            }),
        )
        .route("/docs", get(move || async move { docs_html }))
}
