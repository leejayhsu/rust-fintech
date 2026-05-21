use sqlx::PgPool;

use crate::{
    errors::user_error::UserError,
    models::user::{CreateUserReq, User, UserResp},
    utils,
};

pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<UserResp, UserError> {
    let user = sqlx::query_as!(
        UserResp,
        r#"
        SELECT id, email, role, created_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    user.ok_or(UserError::NotFound)
}

pub async fn create(pool: &PgPool, req: CreateUserReq) -> Result<User, UserError> {
    let password_hash =
        bcrypt::hash(&req.password, bcrypt::DEFAULT_COST).map_err(|_| UserError::PasswordHash)?;

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, password_hash, created_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, role, password_hash, created_at
        "#,
        utils::generate_id("usr"),
        req.email,
        password_hash,
        chrono::Utc::now()
    )
    .fetch_one(pool)
    .await;

    match user {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(UserError::EmailConflict)
        }
        Err(e) => Err(UserError::Database(e)),
    }
}
