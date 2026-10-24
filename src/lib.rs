pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use std::sync::Arc;

use axum::{Router, middleware};
use presentation::http::{AppState, error::request_id_middleware, routes};

pub fn app(repository: Arc<dyn application::port::UserRepository>) -> Router {
    let state = AppState::new(repository);
    Router::new()
        .merge(routes())
        .with_state(state)
        .layer(middleware::from_fn(request_id_middleware))
}
