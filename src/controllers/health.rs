use axum::response::IntoResponse;

pub async fn check() -> impl IntoResponse {
    "OK"
}
