use serde::{Deserialize, Serialize};

/*
UpdateUserDto = request body for PUT /users/:id

Fields are optional so user can update just one thing.
*/
#[derive(Debug, Clone, Serialize, Deserialize, nestforge::Validate)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}
