use nestforge::{ResourceError, ResourceService};

use crate::users::dto::{CreateUserDto, UpdateUserDto, UserDto};

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
    service.all()
}

pub fn get_user(service: &UsersService, id: u64) -> Option<UserDto> {
    service.get(id)
}

pub fn users_count(service: &UsersService) -> usize {
    service.count()
}

pub fn user_exists(service: &UsersService, id: u64) -> bool {
    service.exists(id)
}

pub fn create_user(service: &UsersService, dto: CreateUserDto) -> Result<UserDto, ResourceError> {
    service.create(dto)
}

pub fn update_user(
    service: &UsersService,
    id: u64,
    dto: UpdateUserDto,
) -> Result<Option<UserDto>, ResourceError> {
    service.update(id, dto)
}

pub fn replace_user(
    service: &UsersService,
    id: u64,
    dto: CreateUserDto,
) -> Result<Option<UserDto>, ResourceError> {
    service.replace(id, dto)
}

pub fn delete_user(service: &UsersService, id: u64) -> Option<UserDto> {
    service.delete(id)
}
