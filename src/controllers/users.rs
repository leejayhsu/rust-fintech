use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    errors::{self, user_error::UserError},
    models::user::{CreateUserReq, UserResp},
    services::users as user_service,
};

pub async fn get(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match user_service::find_by_id(&pool, &id).await {
        Ok(user) => errors::success(user),
        Err(UserError::NotFound) => errors::error(
            StatusCode::NOT_FOUND,
            UserError::NotFound.code(),
            &UserError::NotFound.desc(),
        ),
        Err(UserError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = UserError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(e) => {
            tracing::error!("unhandled error: {e}");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, e.code(), &e.desc())
        }
    }
}

pub async fn create(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserReq>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "20005",
            &format!("validation failed: {e}"),
        );
    }

    match user_service::create(&pool, body).await {
        Ok(user) => errors::success(UserResp::from(user)),
        Err(UserError::EmailConflict) => errors::error(
            StatusCode::CONFLICT,
            UserError::EmailConflict.code(),
            &UserError::EmailConflict.desc(),
        ),
        Err(UserError::PasswordHash) => {
            tracing::error!("password hashing failed");
            let err = UserError::PasswordHash;
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(UserError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = UserError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(e) => {
            tracing::error!("unhandled error: {e}");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, e.code(), &e.desc())
        }
    }
}
