use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UsersCountDto {
    pub total: usize,
}
