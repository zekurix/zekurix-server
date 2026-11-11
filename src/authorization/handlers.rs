use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderName, StatusCode, header},
    response::AppendHeaders,
};

use crate::Application;
use crate::error::Result;
use crate::router::{ApiJson, ApiPath, Pagination};

use super::RoleId;
use super::dto::{PermissionsResponse, RoleRequest, RoleResponse, RolesResponse};

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
    ApiJson(params): ApiJson<RoleRequest>,
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

pub async fn get_role(
    State(application): State<Arc<Application>>,
    ApiPath(id): ApiPath<RoleId>,
) -> Result<Json<RoleResponse>> {
    let role = application.services.authorization.get_role(id).await?;

    Ok(Json(role.into()))
}

pub async fn list_roles(
    State(application): State<Arc<Application>>,
    pagination: Pagination,
) -> Result<Json<RolesResponse>> {
    let roles = application
        .services
        .authorization
        .list_roles(pagination)
        .await?;

    Ok(Json(roles.into()))
}

pub async fn update_role(
    State(application): State<Arc<Application>>,
    ApiPath(id): ApiPath<RoleId>,
    ApiJson(params): ApiJson<RoleRequest>,
) -> Result<Json<RoleResponse>> {
    let role = application
        .services
        .authorization
        .update_role(id, params.name, params.permissions)
        .await?;

    Ok(Json(role.into()))
}

pub async fn delete_role(
    State(application): State<Arc<Application>>,
    ApiPath(id): ApiPath<RoleId>,
) -> Result<StatusCode> {
    application.services.authorization.delete_role(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
