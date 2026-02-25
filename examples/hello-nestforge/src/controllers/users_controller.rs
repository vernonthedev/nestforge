use axum::{extract::State, response::Json, routing::get, Router};
use nestforge::{Container, ControllerDefinition, HttpException};

use crate::{dto::UserDto, services::UsersService};

/*
UsersController = handles "/users"
*/
pub struct UsersController;

impl ControllerDefinition for UsersController {
    fn router() -> Router<Container> {
        Router::new().route("/users", get(Self::list_users))
    }
}

impl UsersController {
    /*
    Flow:
    - get Container from state
    - resolve UsersService from DI
    - return JSON list
    - return HttpException if service isn't registered
    */
    async fn list_users(
        State(container): State<Container>,
    ) -> Result<Json<Vec<UserDto>>, HttpException> {
        let service = container.resolve::<UsersService>().map_err(|_| {
            HttpException::internal_server_error(
                "[UsersController] UsersService is not registered in the container",
            )
        })?;

        Ok(Json(service.find_all()))
    }
}