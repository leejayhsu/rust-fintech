use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerJournalEntryError {
    #[error("account not found")]
    AccountNotFound,

    #[error("account balance not found for currency")]
    BalanceNotFound,

    #[error("insufficient funds")]
    InsufficientFunds,

    #[error("invalid journal entry")]
    InvalidJournalEntry,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl LedgerJournalEntryError {
    pub fn code(&self) -> &'static str {
        match self {
            LedgerJournalEntryError::AccountNotFound => "40001",
            LedgerJournalEntryError::BalanceNotFound => "40008",
            LedgerJournalEntryError::InsufficientFunds => "40006",
            LedgerJournalEntryError::InvalidJournalEntry => "40011",
            LedgerJournalEntryError::Database(_) => "40005",
        }
    }

    pub fn desc(&self) -> String {
        self.to_string()
    }
}
