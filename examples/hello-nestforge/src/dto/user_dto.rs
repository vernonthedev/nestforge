/*
UserDto = shape returned to the client as JSON

For now this also acts as our in-memory entity model.
Later you can split Entity vs DTO if you want.
*/
#[nestforge::dto]
pub struct UserDto {
    pub id: u64,
    pub name: String,
    pub email: String,
}

nestforge::impl_identifiable!(UserDto, id);
