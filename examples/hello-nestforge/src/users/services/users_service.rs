use nestforge::{injectable, ResourceError, ResourceService};

use crate::users::dto::{CreateUserDto, UpdateUserDto, UserDto};

#[injectable(factory = build_users_service)]
pub struct UsersService {
    store: ResourceService<UserDto>,
}

fn build_users_service() -> UsersService {
    UsersService {
        store: ResourceService::with_seed(vec![
            UserDto {
                id: 1,
                name: "John Doe".to_string(),
                email: "john.doe@example.com".to_string(),
            },
            UserDto {
                id: 2,
                name: "Jane Doe".to_string(),
                email: "jane.doe@example.com".to_string(),
            },
        ]),
    }
}

impl UsersService {
    pub fn list(&self) -> Vec<UserDto> {
        self.store.all()
    }

    pub fn count(&self) -> usize {
        self.store.count()
    }

    pub fn get(&self, id: u64) -> Option<UserDto> {
        self.store.get(id)
    }

    pub fn exists(&self, id: u64) -> bool {
        self.store.exists(id)
    }

    pub fn create(&self, dto: CreateUserDto) -> Result<UserDto, ResourceError> {
        self.store.create(dto)
    }

    pub fn update(&self, id: u64, dto: UpdateUserDto) -> Result<Option<UserDto>, ResourceError> {
        self.store.update(id, dto)
    }

    pub fn replace(&self, id: u64, dto: CreateUserDto) -> Result<Option<UserDto>, ResourceError> {
        self.store.replace(id, dto)
    }

    pub fn delete(&self, id: u64) -> Option<UserDto> {
        self.store.delete(id)
    }
}
