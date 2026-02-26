#[nestforge::dto]
pub struct UserDto {
    pub id: u64,
    pub name: String,
    pub email: String,
}

nestforge::impl_identifiable!(UserDto, id);
