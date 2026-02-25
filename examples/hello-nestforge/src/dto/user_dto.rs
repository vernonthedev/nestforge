use serde::Serialize;

/*
UserDto = shape returned to the client as JSON
*/
#[derive(Debug, Clone, Serialize)]
pub struct UserDto {
    pub id: u64,
    pub name: String,
    pub email: String,
}