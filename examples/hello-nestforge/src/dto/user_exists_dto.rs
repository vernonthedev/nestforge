#[nestforge::dto]
pub struct UserExistsDto {
    pub id: u64,
    pub exists: bool,
}
