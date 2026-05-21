use sqlx::PgPool;

use crate::{errors::auth_error::AuthError, models::user::User};

pub async fn verify_credentials(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<User, AuthError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, role, password_hash, created_at FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::InvalidCredentials)?;

    let valid =
        bcrypt::verify(password, &user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(user)
}
