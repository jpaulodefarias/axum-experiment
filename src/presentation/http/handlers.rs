use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AppState, error::HttpError};
use crate::{
    application::{error::AppError, usecase::UserService},
    domain::model::User,
};

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct HealthResponse {
    status: &'static str,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub(super) struct UserInput {
    name: String,
    email: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserResponse {
    id: Uuid,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

fn service(
    state: &AppState,
) -> UserService<std::sync::Arc<dyn crate::application::port::UserRepository>> {
    UserService {
        repository: state.repository.clone(),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Service is alive", body = HealthResponse))
)]
pub(super) async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "UP" }))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = UserInput,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid request", body = super::error::ErrorBody),
        (status = 409, description = "Email already exists", body = super::error::ErrorBody),
        (status = 500, description = "Internal server error", body = super::error::ErrorBody)
    )
)]
pub(super) async fn create_user(
    State(state): State<AppState>,
    Extension(request_id): Extension<String>,
    payload: Result<Json<UserInput>, axum::extract::rejection::JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let Json(input) = payload.map_err(|_| invalid_json(&request_id))?;
    let user = service(&state)
        .create(input.name, input.email)
        .await
        .map_err(|error| HttpError {
            error,
            trace_id: request_id.clone(),
        })?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses(
        (status = 200, description = "Users listed", body = [UserResponse]),
        (status = 500, description = "Internal server error", body = super::error::ErrorBody)
    )
)]
pub(super) async fn list_users(
    State(state): State<AppState>,
    Extension(request_id): Extension<String>,
) -> Result<Json<Vec<UserResponse>>, HttpError> {
    let users = service(&state).list().await.map_err(|error| HttpError {
        error,
        trace_id: request_id,
    })?;
    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 400, description = "Invalid user ID", body = super::error::ErrorBody),
        (status = 404, description = "User not found", body = super::error::ErrorBody),
        (status = 500, description = "Internal server error", body = super::error::ErrorBody)
    )
)]
pub(super) async fn get_user(
    State(state): State<AppState>,
    Extension(request_id): Extension<String>,
    Path(raw_id): Path<String>,
) -> Result<Json<UserResponse>, HttpError> {
    let id = parse_id(&raw_id, &request_id)?;
    let user = service(&state).get(id).await.map_err(|error| HttpError {
        error,
        trace_id: request_id,
    })?;
    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "User UUID")),
    request_body = UserInput,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 400, description = "Invalid request", body = super::error::ErrorBody),
        (status = 404, description = "User not found", body = super::error::ErrorBody),
        (status = 409, description = "Email already exists", body = super::error::ErrorBody),
        (status = 500, description = "Internal server error", body = super::error::ErrorBody)
    )
)]
pub(super) async fn update_user(
    State(state): State<AppState>,
    Extension(request_id): Extension<String>,
    Path(raw_id): Path<String>,
    payload: Result<Json<UserInput>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<UserResponse>, HttpError> {
    let id = parse_id(&raw_id, &request_id)?;
    let Json(input) = payload.map_err(|_| invalid_json(&request_id))?;
    let user = service(&state)
        .update(id, input.name, input.email)
        .await
        .map_err(|error| HttpError {
            error,
            trace_id: request_id,
        })?;
    Ok(Json(UserResponse::from(user)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 400, description = "Invalid user ID", body = super::error::ErrorBody),
        (status = 404, description = "User not found", body = super::error::ErrorBody),
        (status = 500, description = "Internal server error", body = super::error::ErrorBody)
    )
)]
pub(super) async fn delete_user(
    State(state): State<AppState>,
    Extension(request_id): Extension<String>,
    Path(raw_id): Path<String>,
) -> Result<StatusCode, HttpError> {
    let id = parse_id(&raw_id, &request_id)?;
    service(&state)
        .delete(id)
        .await
        .map_err(|error| HttpError {
            error,
            trace_id: request_id,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/users",
    tag = "users",
    responses(
        (status = 204, description = "All users deleted"),
        (status = 500, description = "Internal server error", body = super::error::ErrorBody)
    )
)]
pub(super) async fn delete_users(
    State(state): State<AppState>,
    Extension(request_id): Extension<String>,
) -> Result<StatusCode, HttpError> {
    service(&state)
        .delete_all()
        .await
        .map_err(|error| HttpError {
            error,
            trace_id: request_id,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_id(raw_id: &str, request_id: &str) -> Result<Uuid, HttpError> {
    Uuid::parse_str(raw_id).map_err(|_| HttpError {
        error: AppError::Validation {
            message: "id must be a UUID".into(),
        },
        trace_id: request_id.to_owned(),
    })
}

fn invalid_json(request_id: &str) -> HttpError {
    HttpError {
        error: AppError::Validation {
            message: "invalid JSON request body".into(),
        },
        trace_id: request_id.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::error::AppError;
    use axum::response::IntoResponse;

    #[test]
    fn not_found_error_uses_standard_wire_format() {
        let response = HttpError {
            error: AppError::NotFound,
            trace_id: "request-id".into(),
        }
        .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(response.headers().contains_key("x-request-id"));
    }
}
