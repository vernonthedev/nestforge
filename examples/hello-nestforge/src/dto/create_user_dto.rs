/*
CreateUserDto = request body for POST /users
Deserialize allows axum to parse JSON into this struct.
*/
#[nestforge::dto]
pub struct CreateUserDto {
    #[validate(required)]
    pub name: String,
    #[validate(required, email)]
    pub email: String,
}
