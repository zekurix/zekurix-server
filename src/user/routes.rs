use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use crate::application::Application;
use super::handlers;

pub fn router() -> Router<Arc<Application>> {
    Router::new()
        .route("/", post(handlers::create_user))
        .route("/{id}", get(handlers::get_user))
}

