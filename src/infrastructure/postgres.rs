use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgDatabaseError};
use uuid::Uuid;

use crate::{
    application::{error::AppError, port::UserRepository},
    domain::model::User,
};

pub struct PgUserRepository {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<User, AppError> {
        let result = sqlx::query!(
            "INSERT INTO users (id, name, email, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id, name, email, created_at, updated_at",
            user.id, user.name, user.email, user.created_at, user.updated_at
        ).fetch_one(&self.pool).await;
        match result {
            Ok(row) => Ok(User {
                id: row.id,
                name: row.name,
                email: row.email,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }),
            Err(error) => Err(map_db_error(error)),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let row = sqlx::query!(
            "SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.map(|row| User {
            id: row.id,
            name: row.name,
            email: row.email,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    async fn list(&self) -> Result<Vec<User>, AppError> {
        let rows = sqlx::query!(
            "SELECT id, name, email, created_at, updated_at FROM users ORDER BY created_at, id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(rows
            .into_iter()
            .map(|row| User {
                id: row.id,
                name: row.name,
                email: row.email,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn update(&self, user: &User) -> Result<Option<User>, AppError> {
        let row = sqlx::query!("UPDATE users SET name = $2, email = $3, updated_at = $4 WHERE id = $1 RETURNING id, name, email, created_at, updated_at", user.id, user.name, user.email, user.updated_at)
            .fetch_optional(&self.pool).await.map_err(map_db_error)?;
        Ok(row.map(|row| User {
            id: row.id,
            name: row.name,
            email: row.email,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_all(&self) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM users")
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error
        && let Some(pg_error) = database_error.try_downcast_ref::<PgDatabaseError>()
        && pg_error.code() == "23505"
    {
        return AppError::Conflict;
    }
    tracing::error!(error = %error, "database operation failed");
    AppError::Internal
}
