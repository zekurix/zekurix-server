use axum::Json;
use utoipa::OpenApi as _;

use super::ApiDoc;

pub async fn get_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
