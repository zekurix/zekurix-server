use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::Application;

use super::handlers;

pub fn permissions_router() -> Router<Arc<Application>> {
    Router::new().route("/", get(handlers::list_permissions))
}

pub fn roles_router() -> Router<Arc<Application>> {
    Router::new()
        .route("/", post(handlers::create_role).get(handlers::list_roles))
        .route(
            "/{id}",
            get(handlers::get_role)
                .put(handlers::update_role)
                .delete(handlers::delete_role),
        )
}
