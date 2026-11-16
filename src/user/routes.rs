use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
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
        .route("/{id}/roles", get(handlers::list_user_roles))
        .route(
            "/{user_id}/roles/{role_id}",
            put(handlers::assign_role_to_user).delete(handlers::unassign_role_from_user),
        )
        .route("/{id}/permissions", get(handlers::list_user_permissions))
}
