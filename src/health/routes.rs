use std::sync::Arc;

use super::handlers;
use crate::application::Application;
use axum::{Router, routing::get};

pub fn router() -> Router<Arc<Application>> {
    Router::new().route("/", get(handlers::get_health))
}
