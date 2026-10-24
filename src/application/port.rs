use async_trait::async_trait;
use uuid::Uuid;

use crate::{application::error::AppError, domain::model::User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn list(&self) -> Result<Vec<User>, AppError>;
    async fn update(&self, user: &User) -> Result<Option<User>, AppError>;
    async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
    async fn delete_all(&self) -> Result<(), AppError>;
}

#[async_trait]
impl<T: UserRepository + ?Sized> UserRepository for std::sync::Arc<T> {
    async fn create(&self, user: &User) -> Result<User, AppError> {
        (**self).create(user).await
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        (**self).find_by_id(id).await
    }
    async fn list(&self) -> Result<Vec<User>, AppError> {
        (**self).list().await
    }
    async fn update(&self, user: &User) -> Result<Option<User>, AppError> {
        (**self).update(user).await
    }
    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        (**self).delete(id).await
    }
    async fn delete_all(&self) -> Result<(), AppError> {
        (**self).delete_all().await
    }
}
