use thiserror::Error;

#[derive(Debug, Error)]
pub enum PartyError {
    #[error("party not found")]
    PartyNotFound,

    #[error("party already exists")]
    PartyAlreadyExists,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl PartyError {
    pub fn code(&self) -> &'static str {
        match self {
            PartyError::PartyNotFound => "50001",
            PartyError::PartyAlreadyExists => "50002",
            PartyError::Database(_) => "50003",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}
