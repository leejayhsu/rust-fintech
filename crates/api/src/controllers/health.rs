use axum::response::IntoResponse;

use crate::errors;

pub async fn check() -> impl IntoResponse {
    errors::success("OK")
}
