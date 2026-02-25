use axum::Json;
use nestforge::{controller, routes, Body, HttpException, Inject, Param};

use crate::dto::{CreateUserDto, UpdateUserDto, UserDto};
use crate::services::UsersService;

#[controller("/users")]
pub struct UsersController;

#[routes]
impl UsersController {
    #[get("/")]
    async fn list(
        users: Inject<UsersService>,
    ) -> Result<Json<Vec<UserDto>>, HttpException> {
        Ok(Json(users.find_all()))
    }

    #[get("/{id}")]
    async fn get_user(
        id: Param<u64>,
        users: Inject<UsersService>,
    ) -> Result<Json<UserDto>, HttpException> {
        let user = users
            .find_by_id(*id)
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", *id)))?;

        Ok(Json(user))
    }

    #[post("/")]
    async fn create(
        users: Inject<UsersService>,
        body: Body<CreateUserDto>,
    ) -> Result<Json<UserDto>, HttpException> {
        body.validate().map_err(HttpException::bad_request)?;
        Ok(Json(users.create(body.into_inner())))
    }

    #[put("/{id}")]
    async fn update(
        id: Param<u64>,
        users: Inject<UsersService>,
        body: Body<UpdateUserDto>,
    ) -> Result<Json<UserDto>, HttpException> {
        body.validate().map_err(HttpException::bad_request)?;

        let updated = users
            .update(*id, body.into_inner())
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", *id)))?;

        Ok(Json(updated))
    }
}