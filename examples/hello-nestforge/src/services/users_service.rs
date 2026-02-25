use nestforge::InMemoryStore;

use crate::dto::{CreateUserDto, UpdateUserDto, UserDto};

/*
UsersService = business logic layer
Now storage internals are handled by NestForge's InMemoryStore.
*/
#[derive(Clone)]
pub struct UsersService {
    store: InMemoryStore<UserDto>,
}

impl UsersService {
    pub fn new() -> Self {
        let seed = vec![
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
        ];

        Self {
            store: InMemoryStore::with_seed(seed),
        }
    }

    pub fn find_all(&self) -> Vec<UserDto> {
        self.store.find_all()
    }

    pub fn find_by_id(&self, id: u64) -> Option<UserDto> {
        self.store.find_by_id(id)
    }

    pub fn create(&self, dto: CreateUserDto) -> UserDto {
        let user = UserDto {
            id: 0, /* framework store sets the real id */
            name: dto.name,
            email: dto.email,
        };

        self.store.create(user)
    }

    pub fn update(&self, id: u64, dto: UpdateUserDto) -> Option<UserDto> {
        self.store.update_by_id(id, |user| {
            if let Some(name) = dto.name.clone() {
                user.name = name;
            }

            if let Some(email) = dto.email.clone() {
                user.email = email;
            }
        })
    }
}