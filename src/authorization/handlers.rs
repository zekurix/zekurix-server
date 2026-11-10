use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderName, StatusCode, header},
    response::AppendHeaders,
};

use crate::Application;
use crate::error::Result;
use crate::router::ApiJson;

use super::dto::{CreateRoleRequest, PermissionsResponse, RoleResponse};

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

pub async fn create_role(
    State(application): State<Arc<Application>>,
    ApiJson(params): ApiJson<CreateRoleRequest>,
) -> Result<(
    StatusCode,
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<RoleResponse>,
)> {
    let role = application
        .services
        .authorization
        .create_role(params.name, params.permissions)
        .await?;

    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/roles/{}", role.id()))]),
        Json(role.into()),
    ))
}
