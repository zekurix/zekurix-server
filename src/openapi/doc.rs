use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::health::handlers::get_health,),
    components(schemas(crate::health::handlers::HealthResponse,))
)]
pub struct ApiDoc;
