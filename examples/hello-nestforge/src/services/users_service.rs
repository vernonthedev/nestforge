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
                email: "vernonthedev@gmail.com".to_string(),
            },
            UserDto {
                id: 2,
                name: "John".to_string(),
                email: "john@mail.com".to_string(),
            },
        ]
    }

    /*
    Find one user by id from our mock list.
    Returns None if not found.
    */
    pub fn find_by_id(&self, id: u64) -> Option<UserDto> {
        self.find_all().into_iter().find(|user| user.id == id)
    }
}