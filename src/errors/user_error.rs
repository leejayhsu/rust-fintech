use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user not found")]
    NotFound,

    #[error("email already in use")]
    EmailConflict,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("password hashing error")]
    PasswordHash,
}
