use nestforge::{ResourceError, ResourceService};

use crate::dto::{CreateUserDto, UpdateUserDto, UserDto};

/*
UsersService = thin alias over NestForge's generic ResourceService.
Built-in methods available directly on this alias:
- find_all()
- find_by_id(id)
- count()
- exists(id)
- create_from(dto)
- update_from(id, dto)
- replace_from(id, dto)
- delete_by_id(id)

*/
pub type UsersService = ResourceService<UserDto>;

pub fn users_service_seed() -> UsersService {
    ResourceService::with_seed(vec![
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
    ])
}

pub fn list_users(service: &UsersService) -> Vec<UserDto> {
    service.find_all()
}

pub fn get_user(service: &UsersService, id: u64) -> Option<UserDto> {
    service.find_by_id(id)
}

pub fn users_count(service: &UsersService) -> usize {
    service.count()
}

pub fn user_exists(service: &UsersService, id: u64) -> bool {
    service.exists(id)
}

pub fn create_user(service: &UsersService, dto: CreateUserDto) -> Result<UserDto, ResourceError> {
    service.create_from(dto)
}

pub fn update_user(
    service: &UsersService,
    id: u64,
    dto: UpdateUserDto,
) -> Result<Option<UserDto>, ResourceError> {
    service.update_from(id, dto)
}

pub fn replace_user(
    service: &UsersService,
    id: u64,
    dto: CreateUserDto,
) -> Result<Option<UserDto>, ResourceError> {
    service.replace_from(id, dto)
}

pub fn delete_user(service: &UsersService, id: u64) -> Option<UserDto> {
    service.delete_by_id(id)
}

#[allow(dead_code)]
pub fn find_user_by_email(service: &UsersService, email: &str) -> Option<UserDto> {
    service
        .find_all()
        .into_iter()
        .find(|user| user.email.eq_ignore_ascii_case(email))
}
