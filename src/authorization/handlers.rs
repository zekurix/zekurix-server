use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderName, StatusCode, header},
    response::AppendHeaders,
};

use crate::Application;
use crate::authorization::permission::{
    RequirePermission, RolesCreate, RolesDelete, RolesRead, RolesUpdate,
};
use crate::error::Result;
use crate::identity::CurrentUserId;
use crate::router::{ApiJson, ApiPath, ApiQuery, PaginationQuery};

use super::dto::{PermissionsResponse, RoleRequest, RoleResponse, RolesPageResponse};
use super::role::{RoleFilter, RoleId, RoleQuery};

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
    CurrentUserId(actor_id): CurrentUserId,
    _: RequirePermission<RolesCreate>,
    ApiJson(params): ApiJson<RoleRequest>,
) -> Result<(
    StatusCode,
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<RoleResponse>,
)> {
    let role = application
        .services
        .authorization
        .create_role(actor_id, params.name, params.permissions)
        .await?;

    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/roles/{}", role.id()))]),
        Json(role.into()),
    ))
}

pub async fn get_role(
    State(application): State<Arc<Application>>,
    _: RequirePermission<RolesRead>,
    ApiPath(id): ApiPath<RoleId>,
) -> Result<Json<RoleResponse>> {
    let role = application.services.authorization.get_role(id).await?;

    Ok(Json(role.into()))
}

pub async fn list_roles(
    State(application): State<Arc<Application>>,
    _: RequirePermission<RolesRead>,
    ApiQuery(query): ApiQuery<RoleQuery>,
    pagination: PaginationQuery,
) -> Result<Json<RolesPageResponse>> {
    let filter = RoleFilter::from(query);

    let roles = application
        .services
        .authorization
        .list_roles(&filter, pagination.into())
        .await?;

    Ok(Json(roles.into()))
}

pub async fn update_role(
    State(application): State<Arc<Application>>,
    CurrentUserId(actor_id): CurrentUserId,
    _: RequirePermission<RolesUpdate>,
    ApiPath(id): ApiPath<RoleId>,
    ApiJson(params): ApiJson<RoleRequest>,
) -> Result<Json<RoleResponse>> {
    let role = application
        .services
        .authorization
        .update_role(actor_id, id, params.name, params.permissions)
        .await?;

    Ok(Json(role.into()))
}

pub async fn delete_role(
    State(application): State<Arc<Application>>,
    _: RequirePermission<RolesDelete>,
    ApiPath(id): ApiPath<RoleId>,
) -> Result<StatusCode> {
    application.services.authorization.delete_role(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
