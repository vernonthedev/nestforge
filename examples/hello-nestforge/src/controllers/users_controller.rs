use axum::{extract::State, response::Json, routing::get, Router};
use nestforge::{Container, ControllerDefinition};

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
    */
    async fn list_users(State(container): State<Container>) -> Json<Vec<UserDto>> {
        let users = match container.resolve::<UsersService>() {
            Ok(service) => service.find_all(),
            Err(_) => Vec::new(),
        };

        Json(users)
    }
}