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

impl UserError {
    pub fn code(&self) -> &'static str {
        match self {
            UserError::NotFound => "20001",
            UserError::EmailConflict => "20002",
            UserError::Database(_) => "20003",
            UserError::PasswordHash => "20004",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}
