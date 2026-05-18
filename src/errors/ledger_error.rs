use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("currency not found")]
    CurrencyNotFound,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl LedgerError {
    pub fn code(&self) -> &'static str {
        match self {
            LedgerError::CurrencyNotFound => "40001",
            LedgerError::Database(_) => "40002",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}
