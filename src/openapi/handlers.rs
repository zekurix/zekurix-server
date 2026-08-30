use axum::{http::header, response::IntoResponse};

const OPENAPI_JSON: &str = include_str!(env!("OPENAPI_JSON_PATH"));

pub async fn get_openapi() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON)
}
