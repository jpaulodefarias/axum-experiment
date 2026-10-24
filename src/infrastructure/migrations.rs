use sea_orm_migration::prelude::*;

mod m0001_create_users {
    use super::*;

    #[derive(DeriveMigrationName)]
    pub struct Migration;

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .get_connection()
                .execute_unprepared(
                    "CREATE TABLE users (\
                        id UUID PRIMARY KEY, \
                        name VARCHAR(200) NOT NULL, \
                        email VARCHAR(320) NOT NULL UNIQUE, \
                        created_at TIMESTAMPTZ NOT NULL, \
                        updated_at TIMESTAMPTZ NOT NULL\
                    )",
                )
                .await?;
            Ok(())
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .drop_table(Table::drop().table(Users::Table).to_owned())
                .await
        }
    }

    #[derive(DeriveIden)]
    enum Users {
        Table,
    }
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m0001_create_users::Migration)]
    }
}
