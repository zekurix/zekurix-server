use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::health::handlers::get_health,
        crate::user::handlers::create_user,
        crate::user::handlers::get_user,
    ),
    components(schemas(
        crate::error::ErrorResponse,
        crate::health::handlers::HealthResponse,
        crate::user::handlers::CreateUserRequest,
        crate::user::handlers::UserResponse,
    ))
)]
pub struct ApiDoc;
