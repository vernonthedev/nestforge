use serde::{Deserialize, Serialize};

/*
UserDto = shape returned to the client as JSON

For now this also acts as our in-memory entity model.
Later you can split Entity vs DTO if you want.
*/
#[derive(Debug, Clone, Serialize, Deserialize, nestforge::Identifiable)]
pub struct UserDto {
    #[id]
    pub id: u64,
    pub name: String,
    pub email: String,
}
