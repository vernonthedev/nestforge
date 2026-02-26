use axum::Json;
use nestforge::{controller, routes, HttpException, Inject, Param, ValidatedBody};

use crate::dto::{CreateUserDto, UpdateUserDto, UserDto, UserExistsDto, UsersCountDto};
use crate::services::{
    create_user, delete_user, get_user, list_users, replace_user, update_user, user_exists,
    users_count, UsersService,
};

#[controller("/users")]
pub struct UsersController;

#[routes]
impl UsersController {
    #[nestforge::get("/")]
    async fn list(users: Inject<UsersService>) -> Result<Json<Vec<UserDto>>, HttpException> {
        Ok(Json(list_users(&users)))
    }

    #[nestforge::get("/count")]
    async fn count(users: Inject<UsersService>) -> Result<Json<UsersCountDto>, HttpException> {
        Ok(Json(UsersCountDto { total: users_count(&users) }))
    }

    #[nestforge::get("/{id}")]
    async fn get_user(
        id: Param<u64>,
        users: Inject<UsersService>,
    ) -> Result<Json<UserDto>, HttpException> {
        let user = get_user(&users, *id)
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", *id)))?;

        Ok(Json(user))
    }

    #[nestforge::post("/")]
    async fn create(
        users: Inject<UsersService>,
        body: ValidatedBody<CreateUserDto>,
    ) -> Result<Json<UserDto>, HttpException> {
        let user = create_user(&users, body.into_inner())
            .map_err(|err| HttpException::bad_request(err.to_string()))?;
        Ok(Json(user))
    }

    #[nestforge::put("/{id}")]
    async fn update(
        id: Param<u64>,
        users: Inject<UsersService>,
        body: ValidatedBody<UpdateUserDto>,
    ) -> Result<Json<UserDto>, HttpException> {
        let updated = update_user(&users, *id, body.into_inner())
            .map_err(|err| HttpException::bad_request(err.to_string()))?
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", *id)))?;

        Ok(Json(updated))
    }

    #[nestforge::put("/{id}/replace")]
    async fn replace(
        id: Param<u64>,
        users: Inject<UsersService>,
        body: ValidatedBody<CreateUserDto>,
    ) -> Result<Json<UserDto>, HttpException> {
        let replaced = replace_user(&users, *id, body.into_inner())
            .map_err(|err| HttpException::bad_request(err.to_string()))?
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", *id)))?;

        Ok(Json(replaced))
    }

    #[nestforge::get("/{id}/exists")]
    async fn exists(
        id: Param<u64>,
        users: Inject<UsersService>,
    ) -> Result<Json<UserExistsDto>, HttpException> {
        Ok(Json(UserExistsDto {
            id: *id,
            exists: user_exists(&users, *id),
        }))
    }

    #[nestforge::delete("/{id}")]
    async fn delete(
        id: Param<u64>,
        users: Inject<UsersService>,
    ) -> Result<Json<UserDto>, HttpException> {
        let deleted = delete_user(&users, *id)
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", *id)))?;

        Ok(Json(deleted))
    }
}
