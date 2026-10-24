use axum::{
    Json,
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use crate::application::error::AppError;

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: &'static str,
    pub trace_id: String,
}

pub struct HttpError {
    pub error: AppError,
    pub trace_id: String,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self.error {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                "User was not found",
            ),
            AppError::Conflict => (
                StatusCode::CONFLICT,
                "USER_ALREADY_EXISTS",
                "A user with this email already exists",
            ),
            AppError::Validation { .. } => (
                StatusCode::BAD_REQUEST,
                "INVALID_REQUEST",
                "The request is invalid",
            ),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred",
            ),
        };
        let mut response = (
            status,
            Json(ErrorBody {
                code,
                message,
                trace_id: self.trace_id,
            }),
        )
            .into_response();
        response.headers_mut().insert(
            "x-request-id",
            HeaderValue::from_str(&Uuid::new_v4().to_string())
                .expect("UUID is a valid header value"),
        );
        response
    }
}

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| Uuid::parse_str(value).is_ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    request.extensions_mut().insert(request_id.clone());
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let started = std::time::Instant::now();
    let mut response = next.run(request).await;
    tracing::info!(request_id = %request_id, method = %method, path = %path,
        status = response.status().as_u16(), duration_ms = started.elapsed().as_millis(),
        "http request completed");
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).expect("UUID is a valid header value"),
    );
    response
}
