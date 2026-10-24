use std::env;

use axum_experiment::{
    application::{error::AppError, port::UserRepository},
    domain::model::User,
    infrastructure::postgres::PgUserRepository,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

// Run with: DATABASE_URL=... cargo test --test postgres -- --ignored
#[tokio::test]
#[ignore = "requires a running PostgreSQL database"]
async fn migration_and_user_repository_crud_and_conflict() {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL is required"))
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repository = PgUserRepository { pool: pool.clone() };
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
    assert!(
        repository
            .list()
            .await
            .unwrap()
            .iter()
            .any(|saved| saved.id == changed.id)
    );
    assert!(repository.delete(changed.id).await.unwrap());
    assert_eq!(repository.find_by_id(changed.id).await.unwrap(), None);
    // Cleanup is scoped to this test's row so it can share a local test database safely.
    pool.close().await;
}
