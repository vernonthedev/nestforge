use std::sync::{Arc, RwLock};

use nestforge::async_graphql::{Context, EmptySubscription, InputObject, Object, Schema, SimpleObject};

#[derive(Clone, Debug, SimpleObject)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Clone)]
pub struct UserStore {
    users: Arc<RwLock<Vec<User>>>,
}

impl UserStore {
    pub fn seeded() -> Self {
        Self {
            users: Arc::new(RwLock::new(vec![
                User {
                    id: 1,
                    name: "Vernon".to_string(),
                    email: "vernon@example.com".to_string(),
                },
                User {
                    id: 2,
                    name: "Sam".to_string(),
                    email: "sam@example.com".to_string(),
                },
            ])),
        }
    }

    pub fn all(&self) -> Vec<User> {
        self.users
            .read()
            .map(|users| users.clone())
            .unwrap_or_default()
    }

    pub fn find(&self, id: u64) -> Option<User> {
        self.users
            .read()
            .ok()
            .and_then(|users| users.iter().find(|user| user.id == id).cloned())
    }

    pub fn create(&self, input: CreateUserInput) -> User {
        let mut users = self.users.write().expect("user store should be writable");
        let next_id = users.last().map(|user| user.id + 1).unwrap_or(1);
        let user = User {
            id: next_id,
            name: input.name,
            email: input.email,
        };
        users.push(user.clone());
        user
    }
}

#[derive(Debug, InputObject)]
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "ok"
    }

    async fn app_name(&self, ctx: &Context<'_>) -> &str {
        ctx.data_unchecked::<String>().as_str()
    }

    async fn users(&self, ctx: &Context<'_>) -> Vec<User> {
        ctx.data_unchecked::<UserStore>().all()
    }

    async fn user(&self, ctx: &Context<'_>, id: u64) -> Option<User> {
        ctx.data_unchecked::<UserStore>().find(id)
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> User {
        ctx.data_unchecked::<UserStore>().create(input)
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(app_name: String) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(UserStore::seeded())
        .data(app_name)
        .finish()
}
