use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("account not found")]
    AccountNotFound,

    #[error("user not found")]
    UserNotFound,

    #[error("currency not found")]
    CurrencyNotFound,

    #[error("duplicate currency balance for account")]
    DuplicateCurrencyBalance,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl LedgerError {
    pub fn code(&self) -> &'static str {
        match self {
            LedgerError::AccountNotFound => "40001",
            LedgerError::UserNotFound => "40002",
            LedgerError::CurrencyNotFound => "40003",
            LedgerError::DuplicateCurrencyBalance => "40004",
            LedgerError::Database(_) => "40005",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}
