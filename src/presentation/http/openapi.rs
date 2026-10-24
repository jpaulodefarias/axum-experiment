use super::{
    error::ErrorBody,
    handlers::{HealthResponse, UserInput, UserResponse},
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        super::handlers::health,
        super::handlers::create_user,
        super::handlers::list_users,
        super::handlers::get_user,
        super::handlers::update_user,
        super::handlers::delete_user,
        super::handlers::delete_users,
    ),
    components(schemas(HealthResponse, UserInput, UserResponse, ErrorBody)),
    tags(
        (name = "health", description = "Liveness endpoint"),
        (name = "users", description = "User management")
    ),
    info(
        title = "Axum Experiment API",
        version = "0.1.0",
        description = "REST API implemented with Rust, Axum, Tokio, and SeaORM."
    )
)]
pub struct ApiDoc;
