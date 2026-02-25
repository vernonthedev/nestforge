use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};
use nestforge::{Container, ControllerDefinition, HttpException, Inject};

use crate::{dto::UserDto, services::UsersService};

/*
UsersController = handles "/users"
*/
pub struct UsersController;

impl ControllerDefinition for UsersController {
    fn router() -> Router<Container> {
        Router::new()
            .route("/users", get(Self::list_users))
            .route("/users/{id}", get(Self::get_user_by_id))
    }
}

impl UsersController {
    /*
    GET /users
    Returns the full user list
    */
    async fn list_users(
        State(container): State<Container>,
    ) -> Result<Json<Vec<UserDto>>, HttpException> {
        let users_service = Inject::<UsersService>::from(&container).map_err(|_| {
            HttpException::internal_server_error(
                "UsersService is not registered in the container",
            )
        })?;

        Ok(Json(users_service.find_all()))
    }

    /*
    GET /users/:id
    Returns a single user or a 404 error
    */
    async fn get_user_by_id(
        Path(id): Path<u64>,
        State(container): State<Container>,
    ) -> Result<Json<UserDto>, HttpException> {
        let users_service = Inject::<UsersService>::from(&container).map_err(|_| {
            HttpException::internal_server_error(
                "UsersService is not registered in the container",
            )
        })?;

        let user = users_service
            .find_by_id(id)
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", id)))?;

        Ok(Json(user))
    }
}