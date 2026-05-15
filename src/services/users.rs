use sqlx::PgPool;

use crate::{
    errors::user_error::UserError,
    models::user::{CreateUserRequest, User, UserPublic},
    utils,
};

pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<UserPublic, UserError> {
    let user = sqlx::query_as::<_, UserPublic>(
        r#"
        SELECT id, email, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    user.ok_or(UserError::NotFound)
}

pub async fn create(pool: &PgPool, req: CreateUserRequest) -> Result<User, UserError> {
    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|_| UserError::PasswordHash)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, password_hash, created_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, password_hash, created_at
        "#,
    )
    .bind(utils::generate_id("usr"))
    .bind(req.email)
    .bind(password_hash)
    .bind(chrono::Utc::now())
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
