#[nestforge::dto]
pub struct UpdateSettingDto {
    pub key: Option<String>,
    pub value: Option<String>,
}
