use std::{future::Future, pin::Pin, sync::Arc};

use nestforge_db::{Db, DbError};
use thiserror::Error;

pub type RepoFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait EntityMeta: Send + Sync + 'static {
    type Id: Send + Sync + Clone + 'static;

    fn table_name() -> &'static str;
    fn id_column() -> &'static str {
        "id"
    }
    fn id_value(&self) -> &Self::Id;
}

pub trait Repo<T>: Send + Sync
where
    T: EntityMeta,
{
    fn find_all(&self) -> RepoFuture<'_, Result<Vec<T>, OrmError>>;
    fn find_by_id(&self, id: T::Id) -> RepoFuture<'_, Result<Option<T>, OrmError>>;
    fn create(&self, entity: T) -> RepoFuture<'_, Result<T, OrmError>>;
    fn update_by_id(&self, id: T::Id, entity: T) -> RepoFuture<'_, Result<T, OrmError>>;
    fn delete_by_id(&self, id: T::Id) -> RepoFuture<'_, Result<(), OrmError>>;
}

#[derive(Debug, Error)]
pub enum OrmError {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    #[error("Repository configuration is incomplete: missing `{operation}` implementation")]
    MissingOperation { operation: &'static str },
}

type FindAllHandler<T> =
    Arc<dyn Fn(&Db) -> RepoFuture<'static, Result<Vec<T>, OrmError>> + Send + Sync>;
type FindByIdHandler<T> =
    Arc<dyn Fn(&Db, <T as EntityMeta>::Id) -> RepoFuture<'static, Result<Option<T>, OrmError>> + Send + Sync>;
type CreateHandler<T> =
    Arc<dyn Fn(&Db, T) -> RepoFuture<'static, Result<T, OrmError>> + Send + Sync>;
type UpdateByIdHandler<T> =
    Arc<dyn Fn(&Db, <T as EntityMeta>::Id, T) -> RepoFuture<'static, Result<T, OrmError>> + Send + Sync>;
type DeleteByIdHandler<T> =
    Arc<dyn Fn(&Db, <T as EntityMeta>::Id) -> RepoFuture<'static, Result<(), OrmError>> + Send + Sync>;

pub struct SqlRepo<T>
where
    T: EntityMeta,
{
    db: Db,
    find_all_handler: FindAllHandler<T>,
    find_by_id_handler: FindByIdHandler<T>,
    create_handler: CreateHandler<T>,
    update_by_id_handler: UpdateByIdHandler<T>,
    delete_by_id_handler: DeleteByIdHandler<T>,
}

impl<T> Clone for SqlRepo<T>
where
    T: EntityMeta,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            find_all_handler: Arc::clone(&self.find_all_handler),
            find_by_id_handler: Arc::clone(&self.find_by_id_handler),
            create_handler: Arc::clone(&self.create_handler),
            update_by_id_handler: Arc::clone(&self.update_by_id_handler),
            delete_by_id_handler: Arc::clone(&self.delete_by_id_handler),
        }
    }
}

impl<T> Repo<T> for SqlRepo<T>
where
    T: EntityMeta,
{
    fn find_all(&self) -> RepoFuture<'_, Result<Vec<T>, OrmError>> {
        (self.find_all_handler)(&self.db)
    }

    fn find_by_id(&self, id: T::Id) -> RepoFuture<'_, Result<Option<T>, OrmError>> {
        (self.find_by_id_handler)(&self.db, id)
    }

    fn create(&self, entity: T) -> RepoFuture<'_, Result<T, OrmError>> {
        (self.create_handler)(&self.db, entity)
    }

    fn update_by_id(&self, id: T::Id, entity: T) -> RepoFuture<'_, Result<T, OrmError>> {
        (self.update_by_id_handler)(&self.db, id, entity)
    }

    fn delete_by_id(&self, id: T::Id) -> RepoFuture<'_, Result<(), OrmError>> {
        (self.delete_by_id_handler)(&self.db, id)
    }
}

#[derive(Clone)]
pub struct SqlRepoBuilder<T>
where
    T: EntityMeta,
{
    db: Db,
    find_all_handler: Option<FindAllHandler<T>>,
    find_by_id_handler: Option<FindByIdHandler<T>>,
    create_handler: Option<CreateHandler<T>>,
    update_by_id_handler: Option<UpdateByIdHandler<T>>,
    delete_by_id_handler: Option<DeleteByIdHandler<T>>,
}

impl<T> SqlRepoBuilder<T>
where
    T: EntityMeta,
{
    pub fn new(db: Db) -> Self {
        Self {
            db,
            find_all_handler: None,
            find_by_id_handler: None,
            create_handler: None,
            update_by_id_handler: None,
            delete_by_id_handler: None,
        }
    }

    pub fn with_find_all<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Db) -> RepoFuture<'static, Result<Vec<T>, OrmError>> + Send + Sync + 'static,
    {
        self.find_all_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_find_by_id<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Db, T::Id) -> RepoFuture<'static, Result<Option<T>, OrmError>> + Send + Sync + 'static,
    {
        self.find_by_id_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_create<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Db, T) -> RepoFuture<'static, Result<T, OrmError>> + Send + Sync + 'static,
    {
        self.create_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_update_by_id<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Db, T::Id, T) -> RepoFuture<'static, Result<T, OrmError>> + Send + Sync + 'static,
    {
        self.update_by_id_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_delete_by_id<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Db, T::Id) -> RepoFuture<'static, Result<(), OrmError>> + Send + Sync + 'static,
    {
        self.delete_by_id_handler = Some(Arc::new(handler));
        self
    }

    pub fn build(self) -> Result<SqlRepo<T>, OrmError> {
        Ok(SqlRepo {
            db: self.db,
            find_all_handler: self
                .find_all_handler
                .ok_or(OrmError::MissingOperation {
                    operation: "find_all",
                })?,
            find_by_id_handler: self
                .find_by_id_handler
                .ok_or(OrmError::MissingOperation {
                    operation: "find_by_id",
                })?,
            create_handler: self.create_handler.ok_or(OrmError::MissingOperation {
                operation: "create",
            })?,
            update_by_id_handler: self
                .update_by_id_handler
                .ok_or(OrmError::MissingOperation {
                    operation: "update_by_id",
                })?,
            delete_by_id_handler: self
                .delete_by_id_handler
                .ok_or(OrmError::MissingOperation {
                    operation: "delete_by_id",
                })?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nestforge_db::DbConfig;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct User {
        id: i64,
        name: String,
    }

    impl EntityMeta for User {
        type Id = i64;

        fn table_name() -> &'static str {
            "users"
        }

        fn id_value(&self) -> &Self::Id {
            &self.id
        }
    }

    #[tokio::test]
    async fn builder_requires_all_handlers() {
        let db = Db::connect_lazy(DbConfig::postgres_local("postgres")).expect("lazy db");
        let err = match SqlRepoBuilder::<User>::new(db).build() {
            Ok(_) => panic!("incomplete builder should fail"),
            Err(err) => err,
        };

        assert!(matches!(err, OrmError::MissingOperation { .. }));
    }

    #[tokio::test]
    async fn repo_delegates_to_configured_handlers() {
        let db = Db::connect_lazy(DbConfig::postgres_local("postgres")).expect("lazy db");
        let repo = SqlRepoBuilder::<User>::new(db)
            .with_find_all(|_| {
                Box::pin(async {
                    Ok(vec![User {
                        id: 1,
                        name: "Vernon".to_string(),
                    }])
                })
            })
            .with_find_by_id(|_, id| {
                Box::pin(async move {
                    Ok(Some(User {
                        id,
                        name: "Vernon".to_string(),
                    }))
                })
            })
            .with_create(|_, entity| Box::pin(async move { Ok(entity) }))
            .with_update_by_id(|_, id, mut entity| {
                Box::pin(async move {
                    entity.id = id;
                    Ok(entity)
                })
            })
            .with_delete_by_id(|_, _| Box::pin(async { Ok(()) }))
            .build()
            .expect("builder should succeed");

        let all = repo.find_all().await.expect("find_all");
        assert_eq!(all.len(), 1);

        let one = repo.find_by_id(1).await.expect("find_by_id");
        assert!(one.is_some());
    }
}
