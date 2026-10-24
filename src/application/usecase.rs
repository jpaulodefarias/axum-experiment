use uuid::Uuid;

use crate::{
    application::{error::AppError, port::UserRepository},
    domain::model::User,
};

pub struct UserService<R> {
    pub repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub async fn create(&self, name: String, email: String) -> Result<User, AppError> {
        validate(&name, &email)?;
        self.repository.create(&User::new(name, email)).await
    }

    pub async fn get(&self, id: Uuid) -> Result<User, AppError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn list(&self) -> Result<Vec<User>, AppError> {
        self.repository.list().await
    }

    pub async fn update(&self, id: Uuid, name: String, email: String) -> Result<User, AppError> {
        validate(&name, &email)?;
        let mut user = self.get(id).await?;
        user.update(name, email);
        self.repository
            .update(&user)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    pub async fn delete_all(&self) -> Result<(), AppError> {
        self.repository.delete_all().await
    }
}

fn validate(name: &str, email: &str) -> Result<(), AppError> {
    let name = name.trim();
    if name.is_empty() || name.len() > 200 {
        return Err(AppError::Validation {
            message: "name must contain 1 to 200 characters".into(),
        });
    }
    if email.len() > 320 || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(AppError::Validation {
            message: "email must be a valid email address".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use async_trait::async_trait;

    #[derive(Default)]
    struct FakeRepository {
        users: Mutex<Vec<User>>,
    }

    #[async_trait]
    impl UserRepository for FakeRepository {
        async fn create(&self, user: &User) -> Result<User, AppError> {
            let mut users = self.users.lock().unwrap();
            if users.iter().any(|u| u.email == user.email) {
                return Err(AppError::Conflict);
            }
            users.push(user.clone());
            Ok(user.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.id == id)
                .cloned())
        }
        async fn list(&self) -> Result<Vec<User>, AppError> {
            Ok(self.users.lock().unwrap().clone())
        }
        async fn update(&self, user: &User) -> Result<Option<User>, AppError> {
            let mut users = self.users.lock().unwrap();
            let Some(existing) = users.iter_mut().find(|u| u.id == user.id) else {
                return Ok(None);
            };
            *existing = user.clone();
            Ok(Some(user.clone()))
        }
        async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
            let mut users = self.users.lock().unwrap();
            let len = users.len();
            users.retain(|u| u.id != id);
            Ok(len != users.len())
        }
        async fn delete_all(&self) -> Result<(), AppError> {
            self.users.lock().unwrap().clear();
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_rejects_invalid_input() {
        let service = UserService {
            repository: FakeRepository::default(),
        };
        assert!(matches!(
            service.create("  ".into(), "not-email".into()).await,
            Err(AppError::Validation { .. })
        ));
    }

    #[tokio::test]
    async fn creates_and_reads_user() {
        let service = UserService {
            repository: FakeRepository::default(),
        };
        let user = service
            .create("Ada".into(), "ada@example.com".into())
            .await
            .unwrap();
        assert_eq!(service.get(user.id).await.unwrap(), user);
    }
}
