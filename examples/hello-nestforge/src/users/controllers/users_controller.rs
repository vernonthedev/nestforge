use axum::Json;
use nestforge::{
    ApiResult, Inject, List, OptionHttpExt, Param, ResultHttpExt, ValidatedBody, controller,
    routes,
};

use crate::users::{
    dto::{CreateUserDto, UpdateUserDto, UserDto, UserExistsDto, UsersCountDto},
    services::{
        UsersService, create_user, delete_user, get_user, list_users, replace_user, update_user,
        user_exists, users_count,
    },
};

#[controller("/users")]
pub struct UsersController;

#[routes]
impl UsersController {
    #[nestforge::get("/")]
    #[nestforge::version("1")]
    async fn list(users: Inject<UsersService>) -> ApiResult<List<UserDto>> {
        Ok(Json(list_users(users.as_ref())))
    }

    #[nestforge::get("/count")]
    #[nestforge::version("1")]
    async fn count(users: Inject<UsersService>) -> ApiResult<UsersCountDto> {
        Ok(Json(UsersCountDto {
            total: users_count(users.as_ref()),
        }))
    }

    #[nestforge::get("/{id}")]
    #[nestforge::version("1")]
    #[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
    async fn get_user(id: Param<u64>, users: Inject<UsersService>) -> ApiResult<UserDto> {
        let id = id.value();
        let user = get_user(users.as_ref(), id).or_not_found_id("User", id)?;
        Ok(Json(user))
    }

    #[nestforge::post("/")]
    #[nestforge::version("1")]
    #[nestforge::use_guard(crate::guards::AllowAllGuard)]
    #[nestforge::use_interceptor(crate::interceptors::LoggingInterceptor)]
    async fn create(
        users: Inject<UsersService>,
        body: ValidatedBody<CreateUserDto>,
    ) -> ApiResult<UserDto> {
        let user = create_user(users.as_ref(), body.value()).or_bad_request()?;
        Ok(Json(user))
    }

    #[nestforge::put("/{id}")]
    #[nestforge::version("1")]
    #[nestforge::use_guard(crate::guards::AllowAllGuard)]
    #[nestforge::use_interceptor(crate::interceptors::LoggingInterceptor)]
    #[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
    async fn update(
        id: Param<u64>,
        users: Inject<UsersService>,
        body: ValidatedBody<UpdateUserDto>,
    ) -> ApiResult<UserDto> {
        let id = id.value();
        let updated = update_user(users.as_ref(), id, body.value())
            .or_bad_request()?
            .or_not_found_id("User", id)?;
        Ok(Json(updated))
    }

    #[nestforge::put("/{id}/replace")]
    #[nestforge::version("1")]
    #[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
    async fn replace(
        id: Param<u64>,
        users: Inject<UsersService>,
        body: ValidatedBody<CreateUserDto>,
    ) -> ApiResult<UserDto> {
        let id = id.value();
        let replaced = replace_user(users.as_ref(), id, body.value())
            .or_bad_request()?
            .or_not_found_id("User", id)?;
        Ok(Json(replaced))
    }

    #[nestforge::get("/{id}/exists")]
    #[nestforge::version("1")]
    #[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
    async fn exists(id: Param<u64>, users: Inject<UsersService>) -> ApiResult<UserExistsDto> {
        let id = id.value();
        Ok(Json(UserExistsDto {
            id,
            exists: user_exists(users.as_ref(), id),
        }))
    }

    #[nestforge::delete("/{id}")]
    #[nestforge::version("1")]
    #[nestforge::use_guard(crate::guards::RequireValidIdGuard)]
    async fn delete(id: Param<u64>, users: Inject<UsersService>) -> ApiResult<UserDto> {
        let id = id.value();
        let deleted = delete_user(users.as_ref(), id).or_not_found_id("User", id)?;
        Ok(Json(deleted))
    }
}
