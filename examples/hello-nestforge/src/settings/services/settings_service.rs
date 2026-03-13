use nestforge::{injectable, ResourceService};

use crate::settings::dto::{CreateSettingDto, SettingDto, UpdateSettingDto};

#[injectable(factory = build_settings_service)]
pub struct SettingsService {
    store: ResourceService<SettingDto>,
}

fn build_settings_service() -> SettingsService {
    SettingsService {
        store: ResourceService::with_seed(vec![
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
        ]),
    }
}

impl SettingsService {
    pub fn list(&self) -> Vec<SettingDto> {
        self.store.all()
    }

    pub fn get(&self, id: u64) -> Option<SettingDto> {
        self.store.get(id)
    }

    pub fn create(
        &self,
        dto: CreateSettingDto,
    ) -> Result<SettingDto, nestforge::ResourceError> {
        self.store.create(dto)
    }

    pub fn update(
        &self,
        id: u64,
        dto: UpdateSettingDto,
    ) -> Result<Option<SettingDto>, nestforge::ResourceError> {
        self.store.update(id, dto)
    }

    pub fn delete(&self, id: u64) -> Option<SettingDto> {
        self.store.delete(id)
    }
}
