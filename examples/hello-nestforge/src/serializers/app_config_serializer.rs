use crate::AppConfig;

#[nestforge::response_dto]
pub struct PublicAppConfigDto {
    pub app_name: String,
}

pub struct AppConfigSerializer;

impl nestforge::ResponseSerializer<AppConfig> for AppConfigSerializer {
    type Output = PublicAppConfigDto;

    fn serialize(value: AppConfig) -> Self::Output {
        PublicAppConfigDto {
            app_name: value.app_name,
        }
    }
}
