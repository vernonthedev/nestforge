use nestforge::Identifiable;
use serde::Serialize;

/*
UserDto = shape returned to the client as JSON

For now this also acts as our in-memory entity model.
Later you can split Entity vs DTO if you want.
*/
#[derive(Debug, Clone, Serialize)]
pub struct UserDto {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl Identifiable for UserDto {
    fn id(&self) -> u64 {
        self.id
    }

    fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}
