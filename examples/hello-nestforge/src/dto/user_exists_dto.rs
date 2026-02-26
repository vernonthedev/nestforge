use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UserExistsDto {
    pub id: u64,
    pub exists: bool,
}
