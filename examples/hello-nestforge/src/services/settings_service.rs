use nestforge::ResourceService;

use crate::dto::{CreateSettingDto, SettingDto, UpdateSettingDto};

use super::AppConfig;

pub type SettingsService = ResourceService<SettingDto>;

pub fn settings_service_from_config(config: &AppConfig) -> SettingsService {
    SettingsService::with_seed(vec![
        SettingDto {
            id: 1,
            key: "app_name".to_string(),
            value: config.app_name.clone(),
        },
        SettingDto {
            id: 2,
            key: "log_level".to_string(),
            value: config.log_level.clone(),
        },
    ])
}

pub fn list_settings(service: &SettingsService) -> Vec<SettingDto> {
    service.all()
}

pub fn get_setting(service: &SettingsService, id: u64) -> Option<SettingDto> {
    service.get(id)
}

pub fn create_setting(service: &SettingsService, dto: CreateSettingDto) -> anyhow::Result<SettingDto> {
    Ok(service.create(dto)?)
}

pub fn update_setting(
    service: &SettingsService,
    id: u64,
    dto: UpdateSettingDto,
) -> anyhow::Result<Option<SettingDto>> {
    Ok(service.update(id, dto)?)
}

pub fn delete_setting(service: &SettingsService, id: u64) -> Option<SettingDto> {
    service.delete(id)
}
