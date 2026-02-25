use std::sync::{Arc, RwLock};

use crate::dto::{CreateUserDto, UserDto};

/*
UsersService = business logic + in-memory storage (for now)

This is still simple, but now POST /users actually persists in app memory.
*/
#[derive(Clone)]
pub struct UsersService {
    users: Arc<RwLock<Vec<UserDto>>>,
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
            users: Arc::new(RwLock::new(seed)),
        }
    }

    pub fn find_all(&self) -> Vec<UserDto> {
        self.users
            .read()
            .map(|users| users.clone())
            .unwrap_or_default()
    }

    pub fn find_by_id(&self, id: u64) -> Option<UserDto> {
        self.users
            .read()
            .ok()
            .and_then(|users| users.iter().find(|u| u.id == id).cloned())
    }

    pub fn create(&self, dto: CreateUserDto) -> UserDto {
        let mut users = self
            .users
            .write()
            .expect("users write lock poisoned");

        let next_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;

        let user = UserDto {
            id: next_id,
            name: dto.name,
            email: dto.email,
        };

        users.push(user.clone());
        user
    }
}