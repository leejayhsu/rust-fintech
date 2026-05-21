use thiserror::Error;

#[derive(Debug, Error)]
pub enum OnboardingError {
    #[error("onboarding not found")]
    NotFound,

    #[error("invalid onboarding status")]
    InvalidStatus,

    #[error("failed to start onboarding workflow")]
    TemporalStartFailed,

    #[error("failed to signal onboarding workflow")]
    TemporalSignalFailed,

    #[allow(dead_code)]
    #[error("kyb rejected")]
    KybRejected,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal onboarding token invalid")]
    InternalUnauthorized,

    #[error("invalid onboarding request")]
    InvalidRequest,
}

impl OnboardingError {
    pub fn code(&self) -> &'static str {
        match self {
            OnboardingError::NotFound => "60001",
            OnboardingError::InvalidStatus => "60002",
            OnboardingError::TemporalStartFailed => "60003",
            OnboardingError::TemporalSignalFailed => "60004",
            OnboardingError::KybRejected => "60005",
            OnboardingError::Database(_) => "60006",
            OnboardingError::InternalUnauthorized => "60007",
            OnboardingError::InvalidRequest => "60008",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::OnboardingError;

    #[test]
    fn temporal_start_failed_has_stable_error_code() {
        let err = OnboardingError::TemporalStartFailed;

        assert_eq!(err.code(), "60003");
    }

    #[test]
    fn internal_unauthorized_has_stable_error_code() {
        let err = OnboardingError::InternalUnauthorized;

        assert_eq!(err.code(), "60007");
    }
}
