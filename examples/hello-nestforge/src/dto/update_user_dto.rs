use nestforge::{Validate, ValidationErrors, ValidationIssue};
use serde::Deserialize;

/*
UpdateUserDto = request body for PUT /users/:id

Fields are optional so user can update just one thing.
*/
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl Validate for UpdateUserDto {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();

        if self.name.is_none() && self.email.is_none() {
            errors.push(ValidationIssue {
                field: "body",
                message: "at least one field is required (name or email)".to_string(),
            });
        }

        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.push(ValidationIssue {
                    field: "name",
                    message: "name cannot be empty".to_string(),
                });
            }
        }

        if let Some(email) = &self.email {
            if email.trim().is_empty() {
                errors.push(ValidationIssue {
                    field: "email",
                    message: "email cannot be empty".to_string(),
                });
            }
            if !email.trim().is_empty() && !email.contains('@') {
                errors.push(ValidationIssue {
                    field: "email",
                    message: "email must be valid".to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors::new(errors))
        }
    }
}
