use std::sync::Arc;

use super::handlers;
use crate::application::Application;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<Arc<Application>> {
    Router::new()
        .route("/", post(handlers::create_user))
        .route("/{id}", get(handlers::get_user))
}
