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

impl CreateUserDto {
    /*
    Manual validation for now (simple and clear).
    */
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name is required".to_string());
        }

        if self.email.trim().is_empty() {
            return Err("email is required".to_string());
        }

        if !self.email.contains('@') {
            return Err("email must be valid".to_string());
        }

        Ok(())
    }
}