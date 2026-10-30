use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};

use crate::Application;

pub async fn get_openapi(State(application): State<Arc<Application>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Body::from(application.services.openapi.document()),
    )
}
