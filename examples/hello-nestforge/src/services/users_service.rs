use crate::dto::UserDto;

/*
UsersService = business logic layer (provider)

Nest-style vibe:
- controller handles HTTP
- service handles business/data logic
*/
#[derive(Clone)]
pub struct UsersService;

impl UsersService {
    pub fn new() -> Self {
        Self
    }

    /*
    Mock data for now (later this can be DB logic)
    */
    pub fn find_all(&self) -> Vec<UserDto> {
        vec![
            UserDto {
                id: 1,
                name: "Vernon".to_string(),
                email: "vernon@example.com".to_string(),
            },
            UserDto {
                id: 2,
                name: "Sam".to_string(),
                email: "sam@example.com".to_string(),
            },
        ]
    }
}