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

impl UpdateUserDto {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_none() && self.email.is_none() {
            return Err("at least one field is required (name or email)".to_string());
        }

        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err("name cannot be empty".to_string());
            }
        }

        if let Some(email) = &self.email {
            if email.trim().is_empty() {
                return Err("email cannot be empty".to_string());
            }
            if !email.contains('@') {
                return Err("email must be valid".to_string());
            }
        }

        Ok(())
    }
}