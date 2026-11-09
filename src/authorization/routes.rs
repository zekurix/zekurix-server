use std::sync::Arc;

use axum::{Router, routing::get};

use crate::Application;

use super::handlers;

pub fn permissions_router() -> Router<Arc<Application>> {
    Router::new().route("/", get(handlers::list_permissions))
}
