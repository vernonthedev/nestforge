use nestforge::ResourceService;

use crate::settings::dto::{CreateSettingDto, SettingDto, UpdateSettingDto};

pub type SettingsService = ResourceService<SettingDto>;

pub fn settings_service_seed() -> SettingsService {
    SettingsService::with_seed(vec![
        SettingDto {
            id: 1,
            key: "app_name".to_string(),
            value: "NestForge".to_string(),
        },
        SettingDto {
            id: 2,
            key: "log_level".to_string(),
            value: "info".to_string(),
        },
    ])
}

pub fn list_settings(service: &SettingsService) -> Vec<SettingDto> {
    service.all()
}

pub fn get_setting(service: &SettingsService, id: u64) -> Option<SettingDto> {
    service.get(id)
}

pub fn create_setting(
    service: &SettingsService,
    dto: CreateSettingDto,
) -> Result<SettingDto, nestforge::ResourceError> {
    service.create(dto)
}

pub fn update_setting(
    service: &SettingsService,
    id: u64,
    dto: UpdateSettingDto,
) -> Result<Option<SettingDto>, nestforge::ResourceError> {
    service.update(id, dto)
}

pub fn delete_setting(service: &SettingsService, id: u64) -> Option<SettingDto> {
    service.delete(id)
}
