use std::fmt::{Display, Formatter};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub field: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationIssue>,
}

impl ValidationErrors {
    pub fn new(errors: Vec<ValidationIssue>) -> Self {
        Self { errors }
    }

    pub fn single(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            errors: vec![ValidationIssue {
                field,
                message: message.into(),
            }],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            return write!(f, "validation failed");
        }

        let summary = self
            .errors
            .iter()
            .map(|issue| format!("{}: {}", issue.field, issue.message))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{summary}")
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}
