use crate::app_config::AppConfig;

#[derive(serde::Serialize)]
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
