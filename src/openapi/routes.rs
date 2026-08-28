use std::sync::Arc;

use axum::{Router, routing::get};

use crate::Application;

use super::handlers;

pub fn router() -> Router<Arc<Application>> {
    Router::new().route("/openapi.json", get(handlers::get_openapi))
}
