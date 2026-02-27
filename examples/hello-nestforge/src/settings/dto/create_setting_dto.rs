#[nestforge::dto]
pub struct CreateSettingDto {
    #[validate(required)]
    pub key: String,
    #[validate(required)]
    pub value: String,
}
