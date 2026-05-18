use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("not authenticated")]
    NotAuthenticated,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            AuthError::NotAuthenticated => "10001",
            AuthError::InvalidCredentials => "10002",
            AuthError::Database(_) => "10003",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}
