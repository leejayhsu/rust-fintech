use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LedgerAccount {
    pub id: String,
    pub pending_balance: Decimal,
    pub available_balance: Decimal,
    pub posted_balance: Decimal,
    pub is_neg_balance_allowed: bool,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLedgerAccountReq {
    #[validate(length(min = 3, max = 3, message = "currency code must be a valid 3-letter ISO code"))]
    pub currency_code: String,

    #[serde(default)]
    pub is_neg_balance_allowed: bool,
}

#[derive(Debug, Serialize)]
pub struct LedgerAccountResp {
    pub id: String,
    pub pending_balance: Decimal,
    pub available_balance: Decimal,
    pub posted_balance: Decimal,
    pub is_neg_balance_allowed: bool,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LedgerAccount> for LedgerAccountResp {
    fn from(account: LedgerAccount) -> Self {
        Self {
            id: account.id,
            pending_balance: account.pending_balance,
            available_balance: account.available_balance,
            posted_balance: account.posted_balance,
            is_neg_balance_allowed: account.is_neg_balance_allowed,
            currency_code: account.currency_code,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}
