use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub mod user_error;

pub fn error_response(status: StatusCode, message: &str) -> axum::response::Response {
    (status, Json(json!({ "error": message }))).into_response()
}
