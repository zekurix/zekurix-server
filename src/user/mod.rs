mod dto;
mod entity;
mod id;
mod postgres_repository;
mod repository;
mod routes;
mod service;

pub mod handlers;

pub use entity::User;
pub use id::UserId;
pub use postgres_repository::PostgresUserRepository;
pub use repository::UserRepository;
pub use routes::router;
pub use service::UserService;

#[cfg(test)]
pub use repository::MockUserRepository;
