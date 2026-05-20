use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;

use crate::{
    errors::{self, user_error::UserError},
    services::users as user_service,
};

pub async fn get(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
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
