use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct LedgerEntry {
    pub id: String,
    pub ledger_journal_entry_id: String,
    pub account_id: String,
    pub currency_code: String,
    pub amount: Decimal,
    pub direction: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
