use std::ops::Deref;

use nestforge::{injectable, ResourceService};

use crate::settings::dto::{CreateSettingDto, SettingDto, UpdateSettingDto};

#[injectable(factory = build_settings_service)]
pub struct SettingsService(ResourceService<SettingDto>);

fn build_settings_service() -> SettingsService {
    SettingsService(ResourceService::with_seed(vec![
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
    ]))
}

impl Deref for SettingsService {
    type Target = ResourceService<SettingDto>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
