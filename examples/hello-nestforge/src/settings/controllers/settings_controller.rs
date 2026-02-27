use axum::Json;
use nestforge::{
    ApiResult, Inject, List, OptionHttpExt, Param, ResultHttpExt, ValidatedBody, controller,
    routes,
};

use crate::app_config::AppConfig;
use crate::settings::{
    dto::{CreateSettingDto, SettingDto, UpdateSettingDto},
    services::{
        SettingsService, create_setting, delete_setting, get_setting, list_settings,
        update_setting,
    },
};

#[controller("/settings")]
pub struct SettingsController;

#[routes]
impl SettingsController {
    #[nestforge::get("/runtime")]
    #[nestforge::version("1")]
    async fn runtime_config(cfg: Inject<AppConfig>) -> ApiResult<SettingDto> {
        Ok(Json(SettingDto {
            id: 0,
            key: "app_name".to_string(),
            value: format!("{} (log_level={})", cfg.app_name, cfg.log_level),
        }))
    }

    #[nestforge::get("/")]
    #[nestforge::version("1")]
    async fn list(service: Inject<SettingsService>) -> ApiResult<List<SettingDto>> {
        Ok(Json(list_settings(service.as_ref())))
    }

    #[nestforge::get("/{id}")]
    #[nestforge::version("1")]
    async fn get_one(id: Param<u64>, service: Inject<SettingsService>) -> ApiResult<SettingDto> {
        let id = id.value();
        let setting = get_setting(service.as_ref(), id).or_not_found_id("Setting", id)?;
        Ok(Json(setting))
    }

    #[nestforge::post("/")]
    #[nestforge::version("1")]
    async fn create(
        service: Inject<SettingsService>,
        body: ValidatedBody<CreateSettingDto>,
    ) -> ApiResult<SettingDto> {
        let setting = create_setting(service.as_ref(), body.value()).or_bad_request()?;
        Ok(Json(setting))
    }

    #[nestforge::put("/{id}")]
    #[nestforge::version("1")]
    async fn update(
        id: Param<u64>,
        service: Inject<SettingsService>,
        body: ValidatedBody<UpdateSettingDto>,
    ) -> ApiResult<SettingDto> {
        let id = id.value();
        let setting = update_setting(service.as_ref(), id, body.value())
            .or_bad_request()?
            .or_not_found_id("Setting", id)?;
        Ok(Json(setting))
    }

    #[nestforge::delete("/{id}")]
    #[nestforge::version("1")]
    async fn delete(id: Param<u64>, service: Inject<SettingsService>) -> ApiResult<SettingDto> {
        let id = id.value();
        let setting = delete_setting(service.as_ref(), id).or_not_found_id("Setting", id)?;
        Ok(Json(setting))
    }
}
