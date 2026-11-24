use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderName, StatusCode, header},
    response::AppendHeaders,
};

use crate::Application;
use crate::authorization::permission::{
    RequirePermission, RolesAssign, RolesRead, RolesUnassign, SelfOrPermission, UsersCreate,
    UsersDelete, UsersRead,
};
use crate::authorization::{PermissionsResponse, RoleId, RoleSummariesResponse};
use crate::error::Result;
use crate::identity::CurrentUserId;
use crate::router::{ApiJson, ApiPath, Pagination};

use super::{
    UserId,
    dto::{CreateUserRequest, UserResponse, UsersResponse},
};

pub async fn create_user(
    State(application): State<Arc<Application>>,
    _: RequirePermission<UsersCreate>,
    ApiJson(params): ApiJson<CreateUserRequest>,
) -> Result<(
    StatusCode,
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<UserResponse>,
)> {
    let user = application
        .services
        .user
        .create(params.identity.issuer, params.identity.subject)
        .await?;

    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/users/{}", user.id))]),
        Json(user.into()),
    ))
}

pub async fn get_current_user(
    State(application): State<Arc<Application>>,
    CurrentUserId(id): CurrentUserId,
) -> Result<Json<UserResponse>> {
    let user = application.services.user.find(id).await?;

    Ok(Json(user.into()))
}

pub async fn get_user(
    State(application): State<Arc<Application>>,
    _: SelfOrPermission<UsersRead>,
    ApiPath(id): ApiPath<UserId>,
) -> Result<Json<UserResponse>> {
    let user = application.services.user.find(id).await?;

    Ok(Json(user.into()))
}

pub async fn list_users(
    State(application): State<Arc<Application>>,
    _: RequirePermission<UsersRead>,
    pagination: Pagination,
) -> Result<Json<UsersResponse>> {
    let users = application.services.user.list(pagination).await?;

    Ok(Json(users.into()))
}

pub async fn delete_user(
    State(application): State<Arc<Application>>,
    _: SelfOrPermission<UsersDelete>,
    ApiPath(id): ApiPath<UserId>,
) -> Result<StatusCode> {
    application.services.user.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_role_to_user(
    State(application): State<Arc<Application>>,
    CurrentUserId(actor_id): CurrentUserId,
    _: RequirePermission<RolesAssign>,
    ApiPath((target_user_id, role_id)): ApiPath<(UserId, RoleId)>,
) -> Result<StatusCode> {
    application
        .services
        .authorization
        .assign_role_to_user(actor_id, target_user_id, role_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_user_roles(
    State(application): State<Arc<Application>>,
    ApiPath(id): ApiPath<UserId>,
) -> Result<Json<RoleSummariesResponse>> {
    let roles = application
        .services
        .authorization
        .list_user_roles(id)
        .await?;

    Ok(Json(roles.into()))
}

pub async fn unassign_role_from_user(
    State(application): State<Arc<Application>>,
    _: RequirePermission<RolesUnassign>,
    ApiPath((user_id, role_id)): ApiPath<(UserId, RoleId)>,
) -> Result<StatusCode> {
    application
        .services
        .authorization
        .unassign_role_from_user(user_id, role_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_user_permissions(
    State(application): State<Arc<Application>>,
    _: SelfOrPermission<RolesRead>,
    ApiPath(id): ApiPath<UserId>,
) -> Result<Json<PermissionsResponse>> {
    let permissions = application
        .services
        .authorization
        .list_user_permissions(id)
        .await?;

    Ok(Json(permissions.into()))
}
