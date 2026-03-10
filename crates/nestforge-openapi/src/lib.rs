use axum::{
    http::header,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use nestforge_core::{OpenApiSchemaComponent, RouteDocumentation};
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
    pub required_roles: Vec<String>,
    pub request_body: Option<Value>,
    pub responses: Vec<OpenApiResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiResponse {
    pub status: u16,
    pub description: String,
    pub schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiDoc {
    pub title: String,
    pub version: String,
    pub routes: Vec<OpenApiRoute>,
    pub components: Vec<OpenApiSchemaComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApiUi {
    Simple,
    SwaggerUi,
    Redoc,
}

#[derive(Debug, Clone)]
pub struct OpenApiConfig {
    pub json_path: String,
    pub yaml_path: String,
    pub docs_path: String,
    pub swagger_ui_path: Option<String>,
    pub redoc_path: Option<String>,
    pub default_ui: OpenApiUi,
}

impl Default for OpenApiConfig {
    fn default() -> Self {
        Self {
            json_path: "/openapi.json".to_string(),
            yaml_path: "/openapi.yaml".to_string(),
            docs_path: "/docs".to_string(),
            swagger_ui_path: Some("/swagger-ui".to_string()),
            redoc_path: Some("/redoc".to_string()),
            default_ui: OpenApiUi::SwaggerUi,
        }
    }
}

impl OpenApiConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_json_path(mut self, path: impl Into<String>) -> Self {
        self.json_path = normalize_path(path.into(), "/openapi.json");
        self
    }

    pub fn with_yaml_path(mut self, path: impl Into<String>) -> Self {
        self.yaml_path = normalize_path(path.into(), "/openapi.yaml");
        self
    }

    pub fn with_docs_path(mut self, path: impl Into<String>) -> Self {
        self.docs_path = normalize_path(path.into(), "/docs");
        self
    }

    pub fn with_swagger_ui_path(mut self, path: impl Into<String>) -> Self {
        self.swagger_ui_path = Some(normalize_path(path.into(), "/swagger-ui"));
        self
    }

    pub fn without_swagger_ui(mut self) -> Self {
        self.swagger_ui_path = None;
        self
    }

    pub fn with_redoc_path(mut self, path: impl Into<String>) -> Self {
        self.redoc_path = Some(normalize_path(path.into(), "/redoc"));
        self
    }

    pub fn without_redoc(mut self) -> Self {
        self.redoc_path = None;
        self
    }

    pub fn with_default_ui(mut self, ui: OpenApiUi) -> Self {
        self.default_ui = ui;
        self
    }
}

impl OpenApiDoc {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            routes: Vec::new(),
            components: Vec::new(),
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
            required_roles: Vec::new(),
            responses: vec![OpenApiResponse {
                status: 200,
                description: "OK".to_string(),
                schema: None,
            }],
            request_body: None,
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
                .iter()
                .map(|route| OpenApiRoute {
                    method: route.method.clone(),
                    path: route.path.clone(),
                    summary: route.summary.clone(),
                    description: route.description.clone(),
                    tags: route.tags.clone(),
                    requires_auth: route.requires_auth,
                    required_roles: route.required_roles.clone(),
                    request_body: route.request_body.clone(),
                    responses: route
                        .responses
                        .iter()
                        .map(|response| OpenApiResponse {
                            status: response.status,
                            description: response.description.clone(),
                            schema: response.schema.clone(),
                        })
                        .collect(),
                })
                .collect(),
            components: collect_schema_components(&routes),
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
                    let mut body = json!({ "description": response.description });
                    if let Some(schema) = &response.schema {
                        body["content"] = json!({
                            "application/json": {
                                "schema": schema
                            }
                        });
                    }

                    (response.status.to_string(), body)
                })
                .collect::<serde_json::Map<String, Value>>();
            let mut operation = json!({
                "summary": route.summary,
                "description": route.description,
                "tags": route.tags,
                "responses": responses,
                "x-required-roles": route.required_roles,
                "security": if route.requires_auth { json!([{"bearerAuth": []}]) } else { json!([]) }
            });

            if let Some(request_body) = &route.request_body {
                operation["requestBody"] = json!({
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": request_body
                        }
                    }
                });
            }
            obj.insert(method, operation);
        }

        let schemas = self
            .components
            .iter()
            .map(|component| (component.name.clone(), component.schema.clone()))
            .collect::<serde_json::Map<String, Value>>();

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
                },
                "schemas": schemas
            },
            "paths": paths
        })
    }

    pub fn to_openapi_yaml(&self) -> String {
        json_value_to_yaml(&self.to_openapi_json(), 0)
    }
}

pub fn docs_router<S>(doc: OpenApiDoc) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    docs_router_with_config(doc, OpenApiConfig::default())
}

pub fn docs_router_with_config<S>(doc: OpenApiDoc, config: OpenApiConfig) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let openapi_json = doc.to_openapi_json();
    let openapi_yaml = doc.to_openapi_yaml();
    let simple_docs = render_simple_docs(&doc, &config);
    let primary_docs = match config.default_ui {
        OpenApiUi::Simple => simple_docs.clone(),
        OpenApiUi::SwaggerUi => render_swagger_ui(
            &doc.title,
            &relative_browser_path(&config.docs_path, &config.json_path),
        ),
        OpenApiUi::Redoc => render_redoc_ui(
            &doc.title,
            &relative_browser_path(&config.docs_path, &config.json_path),
        ),
    };

    let mut router = Router::<S>::new()
        .route(
            &config.json_path,
            get({
                let payload = openapi_json.clone();
                move || async move { Json(payload.clone()) }
            }),
        )
        .route(
            &config.yaml_path,
            get({
                let payload = openapi_yaml.clone();
                move || async move { yaml_response(payload.clone()) }
            }),
        )
        .route(
            &config.docs_path,
            get(move || async move { Html(primary_docs.clone()) }),
        );

    if let Some(path) = &config.swagger_ui_path {
        let swagger_docs =
            render_swagger_ui(&doc.title, &relative_browser_path(path, &config.json_path));
        router = router.route(
            path,
            get({
                let html = swagger_docs.clone();
                move || async move { Html(html.clone()) }
            }),
        );
    }

    if let Some(path) = &config.redoc_path {
        let redoc_docs =
            render_redoc_ui(&doc.title, &relative_browser_path(path, &config.json_path));
        router = router.route(
            path,
            get({
                let html = redoc_docs.clone();
                move || async move { Html(html.clone()) }
            }),
        );
    }

    router
}

fn render_simple_docs(doc: &OpenApiDoc, config: &OpenApiConfig) -> String {
    let routes_html = doc
        .routes
        .iter()
        .map(|route| {
            let summary = route
                .summary
                .clone()
                .unwrap_or_else(|| "No summary".to_string());
            format!(
                "<li><strong>{}</strong> <code>{}</code> - {}</li>",
                route.method, route.path, summary
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let swagger_link = config
        .swagger_ui_path
        .as_ref()
        .map(|path| {
            format!(
                r#"<li><a href="{}">Swagger UI</a></li>"#,
                relative_browser_path(&config.docs_path, path)
            )
        })
        .unwrap_or_default();
    let redoc_link = config
        .redoc_path
        .as_ref()
        .map(|path| {
            format!(
                r#"<li><a href="{}">Redoc</a></li>"#,
                relative_browser_path(&config.docs_path, path)
            )
        })
        .unwrap_or_default();

    format!(
        r#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>{title}</title></head>
<body>
  <h1>{title}</h1>
  <p>OpenAPI JSON is available at <code>{json_path}</code>.</p>
  <p>OpenAPI YAML is available at <code>{yaml_path}</code>.</p>
  <ul>
    {swagger_link}
    {redoc_link}
  </ul>
  <ul>{routes_html}</ul>
</body>
</html>"#,
        title = doc.title,
        json_path = relative_browser_path(&config.docs_path, &config.json_path),
        yaml_path = relative_browser_path(&config.docs_path, &config.yaml_path),
    )
}

fn collect_schema_components(routes: &[RouteDocumentation]) -> Vec<OpenApiSchemaComponent> {
    let mut components = Vec::new();

    for route in routes {
        for component in &route.schema_components {
            if !components
                .iter()
                .any(|existing: &OpenApiSchemaComponent| existing.name == component.name)
            {
                components.push(component.clone());
            }
        }
    }

    components
}

fn render_swagger_ui(title: &str, json_path: &str) -> String {
    format!(
        r##"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>{title} - Swagger UI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.ui = SwaggerUIBundle({{
      url: "{json_path}",
      dom_id: "#swagger-ui",
      deepLinking: true,
      presets: [SwaggerUIBundle.presets.apis],
    }});
  </script>
</body>
</html>"##
    )
}

fn render_redoc_ui(title: &str, json_path: &str) -> String {
    format!(
        r##"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>{title} - Redoc</title>
  <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</head>
<body>
  <redoc spec-url="{json_path}"></redoc>
</body>
</html>"##
    )
}

fn yaml_response(payload: String) -> Response {
    (
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        payload,
    )
        .into_response()
}

fn normalize_path(path: String, default_path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return default_path.to_string();
    }

    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

fn relative_browser_path(from_path: &str, to_path: &str) -> String {
    let from_segments = path_segments(from_path);
    let to_segments = path_segments(to_path);

    let from_dir_len = from_segments.len().saturating_sub(1);
    let common_len = from_segments[..from_dir_len]
        .iter()
        .zip(to_segments.iter())
        .take_while(|(left, right)| left == right)
        .count();

    let mut parts = Vec::new();
    for _ in common_len..from_dir_len {
        parts.push("..".to_string());
    }
    for segment in &to_segments[common_len..] {
        parts.push(segment.clone());
    }

    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn path_segments(path: &str) -> Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect()
}

fn json_value_to_yaml(value: &Value, indent: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => format!("\"{}\"", escape_yaml_string(string)),
        Value::Array(items) => {
            if items.is_empty() {
                return "[]".to_string();
            }

            let indent_str = " ".repeat(indent);
            items
                .iter()
                .map(|item| match item {
                    Value::Object(_) | Value::Array(_) => format!(
                        "{indent_str}-\n{}",
                        indent_multiline(&json_value_to_yaml(item, indent + 2), indent + 2)
                    ),
                    _ => format!("{indent_str}- {}", json_value_to_yaml(item, indent + 2)),
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        Value::Object(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }

            let indent_str = " ".repeat(indent);
            map.iter()
                .map(|(key, value)| match value {
                    Value::Object(_) | Value::Array(_) => format!(
                        "{indent_str}{key}:\n{}",
                        indent_multiline(&json_value_to_yaml(value, indent + 2), indent + 2)
                    ),
                    _ => format!(
                        "{indent_str}{key}: {}",
                        json_value_to_yaml(value, indent + 2)
                    ),
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

fn indent_multiline(value: &str, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    value
        .lines()
        .map(|line| format!("{indent_str}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_yaml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use tower::util::ServiceExt;

    use super::{docs_router_with_config, OpenApiConfig, OpenApiDoc, OpenApiUi};

    #[test]
    fn openapi_doc_exports_yaml() {
        let yaml = OpenApiDoc::new("Test API", "1.0.0")
            .add_route("GET", "/users")
            .to_openapi_yaml();

        assert!(yaml.contains("openapi: \"3.1.0\""));
        assert!(yaml.contains("title: \"Test API\""));
        assert!(yaml.contains("/users:"));
    }

    #[tokio::test]
    async fn docs_router_serves_swagger_and_yaml_endpoints() {
        let doc = OpenApiDoc::new("Test API", "1.0.0").add_route("GET", "/users");
        let app: axum::Router = docs_router_with_config(
            doc,
            OpenApiConfig::new()
                .with_docs_path("/api/docs")
                .with_default_ui(OpenApiUi::SwaggerUi),
        );

        let docs_response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/docs")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("docs request should succeed");

        assert_eq!(docs_response.status(), axum::http::StatusCode::OK);

        let yaml_response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/openapi.yaml")
                    .body(axum::body::Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("yaml request should succeed");

        assert_eq!(yaml_response.status(), axum::http::StatusCode::OK);
    }

    #[test]
    fn relative_browser_path_handles_prefixed_docs_routes() {
        assert_eq!(
            super::relative_browser_path("/docs", "/openapi.json"),
            "openapi.json"
        );
        assert_eq!(
            super::relative_browser_path("/api/docs", "/openapi.json"),
            "../openapi.json"
        );
        assert_eq!(
            super::relative_browser_path("/api/v1/docs", "/api/v1/openapi.json"),
            "openapi.json"
        );
    }
}
