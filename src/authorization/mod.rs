mod dto;
mod handlers;
mod permission;
mod postgres_repository;
mod repository;
mod routes;
mod service;

pub use permission::Permission;
pub use postgres_repository::PostgresAuthorizationRepository;
pub use repository::AuthorizationRepository;
pub use routes::permissions_router;
pub use service::AuthorizationService;

#[cfg(test)]
pub use repository::MockAuthorizationRepository;
