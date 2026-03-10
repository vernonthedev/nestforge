use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenApiSchemaComponent {
    pub name: String,
    pub schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteResponseDocumentation {
    pub status: u16,
    pub description: String,
    pub schema: Option<Value>,
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
    pub request_body: Option<Value>,
    pub schema_components: Vec<OpenApiSchemaComponent>,
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
                schema: None,
            }],
            requires_auth: false,
            required_roles: Vec::new(),
            request_body: None,
            schema_components: Vec::new(),
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

    pub fn with_request_body_schema(mut self, schema: Value) -> Self {
        self.request_body = Some(schema);
        self
    }

    pub fn with_success_response_schema(mut self, schema: Value) -> Self {
        if let Some(response) = self
            .responses
            .iter_mut()
            .find(|response| (200..300).contains(&response.status))
        {
            response.schema = Some(schema);
            return self;
        }

        if let Some(response) = self.responses.first_mut() {
            response.schema = Some(schema);
            return self;
        }

        self.responses.push(RouteResponseDocumentation {
            status: 200,
            description: "OK".to_string(),
            schema: Some(schema),
        });
        self
    }

    pub fn with_schema_components<I>(mut self, components: I) -> Self
    where
        I: IntoIterator<Item = OpenApiSchemaComponent>,
    {
        for component in components {
            if !self
                .schema_components
                .iter()
                .any(|existing| existing.name == component.name)
            {
                self.schema_components.push(component);
            }
        }

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

pub trait OpenApiSchema {
    fn schema_name() -> Option<&'static str> {
        None
    }

    fn schema() -> Value;

    fn schema_or_ref() -> Value {
        match Self::schema_name() {
            Some(name) => json!({ "$ref": format!("#/components/schemas/{name}") }),
            None => Self::schema(),
        }
    }

    fn schema_components() -> Vec<OpenApiSchemaComponent> {
        match Self::schema_name() {
            Some(name) => vec![OpenApiSchemaComponent {
                name: name.to_string(),
                schema: Self::schema(),
            }],
            None => Vec::new(),
        }
    }
}

pub fn openapi_schema_for<T: OpenApiSchema>() -> Value {
    T::schema_or_ref()
}

pub fn openapi_schema_components_for<T: OpenApiSchema>() -> Vec<OpenApiSchemaComponent> {
    T::schema_components()
}

pub fn openapi_array_schema_for<T: OpenApiSchema>() -> Value {
    json!({
        "type": "array",
        "items": T::schema_or_ref(),
    })
}

pub fn openapi_nullable_schema_for<T: OpenApiSchema>() -> Value {
    json!({
        "anyOf": [
            T::schema_or_ref(),
            { "type": "null" }
        ]
    })
}

macro_rules! primitive_openapi_schema {
    ($($ty:ty => $schema:expr),* $(,)?) => {
        $(
            impl OpenApiSchema for $ty {
                fn schema() -> Value {
                    $schema
                }
            }
        )*
    };
}

primitive_openapi_schema!(
    String => json!({ "type": "string" }),
    bool => json!({ "type": "boolean" }),
    u8 => json!({ "type": "integer", "format": "uint8" }),
    u16 => json!({ "type": "integer", "format": "uint16" }),
    u32 => json!({ "type": "integer", "format": "uint32" }),
    u64 => json!({ "type": "integer", "format": "uint64" }),
    usize => json!({ "type": "integer", "format": "uint" }),
    i8 => json!({ "type": "integer", "format": "int8" }),
    i16 => json!({ "type": "integer", "format": "int16" }),
    i32 => json!({ "type": "integer", "format": "int32" }),
    i64 => json!({ "type": "integer", "format": "int64" }),
    isize => json!({ "type": "integer", "format": "int" }),
    f32 => json!({ "type": "number", "format": "float" }),
    f64 => json!({ "type": "number", "format": "double" })
);

impl<T> OpenApiSchema for Vec<T>
where
    T: OpenApiSchema,
{
    fn schema() -> Value {
        openapi_array_schema_for::<T>()
    }

    fn schema_components() -> Vec<OpenApiSchemaComponent> {
        T::schema_components()
    }
}

impl<T> OpenApiSchema for Option<T>
where
    T: OpenApiSchema,
{
    fn schema() -> Value {
        openapi_nullable_schema_for::<T>()
    }

    fn schema_components() -> Vec<OpenApiSchemaComponent> {
        T::schema_components()
    }
}
