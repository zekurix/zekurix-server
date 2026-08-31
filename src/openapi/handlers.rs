use axum::{http::header, response::IntoResponse};

const OPENAPI_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/openapi/generated/openapi.min.json"
));

pub async fn get_openapi() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON)
}
