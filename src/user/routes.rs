use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::Application;

use super::handlers;

pub fn router() -> Router<Arc<Application>> {
    Router::new()
        .route("/", post(handlers::create_user).get(handlers::list_users))
        .route("/me", get(handlers::get_current_user))
        .route(
            "/{id}",
            get(handlers::get_user).delete(handlers::delete_user),
        )
}
