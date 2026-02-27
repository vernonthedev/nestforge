#[nestforge::dto]
pub struct CreateUserDto {
    #[validate(required)]
    pub name: String,
    #[validate(required, email)]
    pub email: String,
}
