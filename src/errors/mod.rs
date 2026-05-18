use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

pub mod auth_error;
pub mod ledger_error;
pub mod user_error;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub desc: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    pub data: Option<T>,
}

pub fn success<T: Serialize>(data: T) -> axum::response::Response {
    Json(ApiResponse {
        success: true,
        error: None,
        data: Some(data),
    })
    .into_response()
}

pub fn error(status: StatusCode, code: &str, desc: &str) -> axum::response::Response {
    let body = ApiResponse::<serde_json::Value> {
        success: false,
        error: Some(ApiError {
            code: code.to_string(),
            desc: desc.to_string(),
        }),
        data: Some(json!(null)),
    };
    (status, Json(body)).into_response()
}
