use serde::{Deserialize, Serialize};

/*
CreateUserDto = request body for POST /users
Deserialize allows axum to parse JSON into this struct.
*/
#[derive(Debug, Clone, Serialize, Deserialize, nestforge::Validate)]
pub struct CreateUserDto {
    #[validate(required)]
    pub name: String,
    #[validate(required, email)]
    pub email: String,
}
