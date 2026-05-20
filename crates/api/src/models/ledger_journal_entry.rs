use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct LedgerJournalEntry {
    pub id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateLedgerJournalEntryLegReq {
    pub account_id: String,
    pub currency_code: String,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateLedgerJournalEntryReq {
    pub debits: Vec<CreateLedgerJournalEntryLegReq>,
    pub credits: Vec<CreateLedgerJournalEntryLegReq>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateExchangeLedgerJournalEntryReq {
    pub debit: CreateLedgerJournalEntryLegReq,
    pub credit: CreateLedgerJournalEntryLegReq,
    pub status: String,
}
