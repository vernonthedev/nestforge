use nestforge::{Validate, ValidationErrors, ValidationIssue};
use serde::Deserialize;

/*
CreateUserDto = request body for POST /users
Deserialize allows axum to parse JSON into this struct.
*/
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
}

impl Validate for CreateUserDto {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push(ValidationIssue {
                field: "name",
                message: "name is required".to_string(),
            });
        }

        if self.email.trim().is_empty() {
            errors.push(ValidationIssue {
                field: "email",
                message: "email is required".to_string(),
            });
        }

        if !self.email.trim().is_empty() && !self.email.contains('@') {
            errors.push(ValidationIssue {
                field: "email",
                message: "email must be valid".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors::new(errors))
        }
    }
}
