use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    errors::{error_response, user_error::UserError},
    models::user::{CreateUserRequest, UserResponse},
    services::users as user_service,
};

pub async fn create(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response();
    }

    match user_service::create(&pool, body).await {
        Ok(user) => (StatusCode::CREATED, Json(UserResponse::from(user))).into_response(),
        Err(UserError::EmailConflict) => {
            error_response(StatusCode::CONFLICT, "email already in use")
        }
        Err(UserError::PasswordHash) => {
            tracing::error!("password hashing failed");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
        Err(UserError::Database(e)) => {
            tracing::error!("db error: {e}");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
        Err(e) => {
            tracing::error!("unhandled error: {e}");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
    }
}
