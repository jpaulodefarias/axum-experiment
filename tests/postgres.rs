use std::env;

use axum_experiment::{
    application::{error::AppError, port::UserRepository},
    domain::model::User,
    infrastructure::{migrations::Migrator, postgres::PgUserRepository},
};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTrait;
use uuid::Uuid;

// Run with: DATABASE_URL=... cargo test --test postgres -- --ignored
#[tokio::test]
#[ignore = "requires a running PostgreSQL database"]
async fn migration_and_user_repository_crud_and_conflict() {
    let mut connection_options =
        ConnectOptions::new(env::var("DATABASE_URL").expect("DATABASE_URL is required"));
    connection_options.max_connections(2);
    let connection = Database::connect(connection_options).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    let repository = PgUserRepository {
        connection: connection.clone(),
    };
    let email = format!("{}@example.com", Uuid::new_v4());
    let user = User::new("Integration User".into(), email);
    let created = repository.create(&user).await.unwrap();
    assert_eq!(
        repository.find_by_id(created.id).await.unwrap(),
        Some(created.clone())
    );
    let duplicate_email = User::new("Another User".into(), user.email.clone());
    assert!(matches!(
        repository.create(&duplicate_email).await,
        Err(AppError::Conflict)
    ));

    let mut changed = created.clone();
    changed.update("Updated User".into(), created.email.clone());
    assert_eq!(
        repository.update(&changed).await.unwrap(),
        Some(changed.clone())
    );
    let second_user = repository
        .create(&User::new(
            "Second Integration User".into(),
            format!("{}@example.com", Uuid::new_v4()),
        ))
        .await
        .unwrap();
    let listed_users = repository.list().await.unwrap();
    assert!(listed_users.iter().any(|saved| saved.id == changed.id));
    assert!(listed_users.iter().any(|saved| saved.id == second_user.id));
    assert!(
        listed_users
            .windows(2)
            .all(|pair| { (pair[0].created_at, pair[0].id) <= (pair[1].created_at, pair[1].id) })
    );
    assert!(repository.delete(changed.id).await.unwrap());
    assert_eq!(repository.find_by_id(changed.id).await.unwrap(), None);
    assert!(repository.delete(second_user.id).await.unwrap());
    // Cleanup is scoped to this test's rows so it can share a test database safely.
    connection.close().await.unwrap();
}
