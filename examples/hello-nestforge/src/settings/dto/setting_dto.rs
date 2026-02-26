#[nestforge::dto]
pub struct SettingDto {
    pub id: u64,
    pub key: String,
    pub value: String,
}

nestforge::impl_identifiable!(SettingDto, id);
