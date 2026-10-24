pub mod error;
mod handlers;
mod openapi;

use std::sync::Arc;

use crate::application::port::UserRepository;
use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<dyn UserRepository>,
}

impl AppState {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route(
            "/api/v1/users",
            get(handlers::list_users)
                .post(handlers::create_user)
                .delete(handlers::delete_users),
        )
        .route(
            "/api/v1/users/{id}",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
}
