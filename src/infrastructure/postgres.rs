use chrono::Utc;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    SqlErr, sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    application::{error::AppError, port::UserRepository},
    domain::model::User,
};

mod entity;

pub struct PgUserRepository {
    pub connection: DatabaseConnection,
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<User, AppError> {
        let model = entity::ActiveModel {
            id: Set(user.id),
            name: Set(user.name.clone()),
            email: Set(user.email.clone()),
            created_at: Set(user.created_at.fixed_offset()),
            updated_at: Set(user.updated_at.fixed_offset()),
        };

        entity::Entity::insert(model)
            .exec_with_returning(&self.connection)
            .await
            .map(Into::into)
            .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        entity::Entity::find_by_id(id)
            .one(&self.connection)
            .await
            .map(|model| model.map(Into::into))
            .map_err(map_db_error)
    }

    async fn list(&self) -> Result<Vec<User>, AppError> {
        entity::Entity::find()
            .order_by_asc(entity::Column::CreatedAt)
            .order_by_asc(entity::Column::Id)
            .all(&self.connection)
            .await
            .map(|models| models.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }

    async fn update(&self, user: &User) -> Result<Option<User>, AppError> {
        let models = entity::Entity::update_many()
            .col_expr(entity::Column::Name, Expr::value(user.name.clone()))
            .col_expr(entity::Column::Email, Expr::value(user.email.clone()))
            .col_expr(
                entity::Column::UpdatedAt,
                Expr::value(user.updated_at.fixed_offset()),
            )
            .filter(entity::Column::Id.eq(user.id))
            .exec_with_returning(&self.connection)
            .await
            .map_err(map_db_error)?;

        Ok(models.into_iter().next().map(Into::into))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        entity::Entity::delete_by_id(id)
            .exec(&self.connection)
            .await
            .map(|result| result.rows_affected > 0)
            .map_err(map_db_error)
    }

    async fn delete_all(&self) -> Result<(), AppError> {
        entity::Entity::delete_many()
            .exec(&self.connection)
            .await
            .map(|_| ())
            .map_err(map_db_error)
    }
}

impl From<entity::Model> for User {
    fn from(model: entity::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            email: model.email,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}

fn map_db_error(error: DbErr) -> AppError {
    if matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) {
        return AppError::Conflict;
    }

    tracing::error!(error = %error, "database operation failed");
    AppError::Internal
}
