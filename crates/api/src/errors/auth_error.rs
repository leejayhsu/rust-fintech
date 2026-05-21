use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("not authenticated")]
    NotAuthenticated,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("admin access required")]
    AdminRequired,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            AuthError::NotAuthenticated => "10001",
            AuthError::InvalidCredentials => "10002",
            AuthError::Database(_) => "10003",
            AuthError::AdminRequired => "10007",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::AuthError;

    #[test]
    fn admin_required_has_stable_error_code() {
        let err = AuthError::AdminRequired;

        assert_eq!(err.code(), "10007");
        assert_eq!(err.desc(), "admin access required");
    }
}
