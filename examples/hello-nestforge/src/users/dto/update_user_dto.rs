#[nestforge::dto]
pub struct UpdateUserDto {
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}
