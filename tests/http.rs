use std::sync::Mutex;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_experiment::{
    app,
    application::{error::AppError, port::UserRepository},
    domain::model::User,
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Default)]
struct MemoryRepository {
    users: Mutex<Vec<User>>,
}

#[async_trait]
impl UserRepository for MemoryRepository {
    async fn create(&self, user: &User) -> Result<User, AppError> {
        let mut users = self.users.lock().unwrap();
        if users.iter().any(|saved| saved.email == user.email) {
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
            .find(|user| user.id == id)
            .cloned())
    }
    async fn list(&self) -> Result<Vec<User>, AppError> {
        Ok(self.users.lock().unwrap().clone())
    }
    async fn update(&self, user: &User) -> Result<Option<User>, AppError> {
        let mut users = self.users.lock().unwrap();
        let Some(saved) = users.iter_mut().find(|saved| saved.id == user.id) else {
            return Ok(None);
        };
        *saved = user.clone();
        Ok(Some(user.clone()))
    }
    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let mut users = self.users.lock().unwrap();
        let length = users.len();
        users.retain(|user| user.id != id);
        Ok(length != users.len())
    }
    async fn delete_all(&self) -> Result<(), AppError> {
        self.users.lock().unwrap().clear();
        Ok(())
    }
}

fn test_app() -> axum::Router {
    app(std::sync::Arc::new(MemoryRepository::default()))
}

async fn request(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    app.oneshot(
        builder
            .body(Body::from(body.unwrap_or_default().to_owned()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn health_returns_up_and_request_id() {
    let response = request(test_app(), "GET", "/health", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], br#"{"status":"UP"}"#);
}

#[tokio::test]
async fn crud_lifecycle_returns_expected_statuses() {
    let app = test_app();
    let created = request(
        app.clone(),
        "POST",
        "/api/v1/users",
        Some(r#"{"name":"Ada","email":"ada@example.com"}"#),
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let body = created.into_body().collect().await.unwrap().to_bytes();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = user["id"].as_str().unwrap();
    let read = request(app.clone(), "GET", &format!("/api/v1/users/{id}"), None).await;
    assert_eq!(read.status(), StatusCode::OK);
    let updated = request(
        app.clone(),
        "PUT",
        &format!("/api/v1/users/{id}"),
        Some(r#"{"name":"Ada Lovelace","email":"ada@example.com"}"#),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(
        request(app.clone(), "DELETE", &format!("/api/v1/users/{id}"), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(app.clone(), "GET", &format!("/api/v1/users/{id}"), None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(app.clone(), "DELETE", "/api/v1/users", None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn validation_errors_use_standard_error_shape() {
    let response = request(
        test_app(),
        "POST",
        "/api/v1/users",
        Some(r#"{"name":" ","email":"bad"}"#),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(response.headers().contains_key("x-request-id"));
    let request_id = response.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let body: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["code"], "INVALID_REQUEST");
    assert_eq!(body["traceId"], request_id);
}

#[tokio::test]
async fn malformed_json_uses_standard_error_shape() {
    let response = request(test_app(), "POST", "/api/v1/users", Some("{")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["code"], "INVALID_REQUEST");
}
