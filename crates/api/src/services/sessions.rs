use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::{
    errors::auth_error::AuthError,
    models::{session::Session, user::UserResp},
    utils,
};

pub async fn create(pool: &PgPool, user_id: &str) -> Result<String, AuthError> {
    let token = generate_session_token();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query_as!(
        Session,
        r#"
        INSERT INTO sessions (id, user_id, token, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, token, expires_at, created_at
        "#,
        utils::generate_id("sess"),
        user_id,
        token,
        expires_at,
        Utc::now()
    )
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn validate_session(pool: &PgPool, token: &str) -> Result<UserResp, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT u.id, u.email, u.created_at
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.token = $1 AND s.expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(UserResp {
            id: r.id,
            email: r.email,
            created_at: r.created_at,
        }),
        None => Err(AuthError::NotAuthenticated),
    }
}

pub async fn delete_by_token(pool: &PgPool, token: &str) -> Result<(), AuthError> {
    sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
        .execute(pool)
        .await?;
    Ok(())
}

fn generate_session_token() -> String {
    nanoid::nanoid!(32)
}
