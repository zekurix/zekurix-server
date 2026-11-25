mod bootstrap;
mod dto;
mod handlers;
pub mod permission;
mod postgres_repository;
mod repository;
mod role;
mod routes;
mod service;

pub use bootstrap::{
    AuthorizationBootstrapRepository, AuthorizationBootstrapService,
    PostgresAuthorizationBootstrapRepository,
};
pub use dto::{PermissionsResponse, RoleSummariesResponse, RoleSummaryResponse};
pub use permission::{Permission, RequirePermission, SelfOrPermission};
pub use postgres_repository::PostgresAuthorizationRepository;
pub use repository::AuthorizationRepository;
pub use role::{Role, RoleFilter, RoleId, RoleName, RoleSummary};
pub use routes::{permissions_router, roles_router};
pub use service::AuthorizationService;

#[cfg(test)]
pub use repository::MockAuthorizationRepository;
