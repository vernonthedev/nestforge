use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use nestforge::{Container, ControllerDefinition, HttpException, Inject};

use crate::dto::{CreateUserDto, UserDto};
use crate::services::UsersService;

pub struct UsersController;

impl ControllerDefinition for UsersController {
    fn router() -> Router<Container> {
        Router::new()
            .route("/users", get(Self::list_users).post(Self::create_user))
            .route("/users/{id}", get(Self::get_user_by_id))
    }
}

impl UsersController {
    async fn list_users(
        State(container): State<Container>,
    ) -> Result<Json<Vec<UserDto>>, HttpException> {
        let users_service = Inject::<UsersService>::from(&container).map_err(|_| {
            HttpException::internal_server_error("UsersService is not registered in the container")
        })?;

        Ok(Json(users_service.find_all()))
    }

    async fn get_user_by_id(
        Path(id): Path<u64>,
        State(container): State<Container>,
    ) -> Result<Json<UserDto>, HttpException> {
        let users_service = Inject::<UsersService>::from(&container).map_err(|_| {
            HttpException::internal_server_error("UsersService is not registered in the container")
        })?;

        let user = users_service
            .find_by_id(id)
            .ok_or_else(|| HttpException::not_found(format!("User with id {} not found", id)))?;

        Ok(Json(user))
    }

    /*
    POST /users
    - parse JSON body
    - validate DTO
    - call service
    */
    async fn create_user(
        State(container): State<Container>,
        Json(dto): Json<CreateUserDto>,
    ) -> Result<Json<UserDto>, HttpException> {
        dto.validate()
            .map_err(HttpException::bad_request)?;

        let users_service = Inject::<UsersService>::from(&container).map_err(|_| {
            HttpException::internal_server_error("UsersService is not registered in the container")
        })?;

        let created = users_service.create(dto);

        Ok(Json(created))
    }
}