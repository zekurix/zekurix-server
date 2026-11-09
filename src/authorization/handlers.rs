use std::sync::Arc;

use axum::{Json, extract::State};

use crate::Application;
use crate::error::Result;

use super::dto::PermissionsResponse;

pub async fn list_permissions(
    State(application): State<Arc<Application>>,
) -> Result<Json<PermissionsResponse>> {
    let permissions = application
        .services
        .authorization
        .list_permissions()
        .await?;

    Ok(Json(permissions.into()))
}
