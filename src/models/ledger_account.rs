use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LedgerAccount {
    pub id: String,
    pub owner_id: String,
    pub name: Option<String>,
    pub is_neg_balance_allowed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LedgerAccountBalance {
    pub account_id: String,
    pub currency_code: String,
    pub pending_balance: Decimal,
    pub available_balance: Decimal,
    pub posted_balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLedgerAccountReq {
    #[validate(length(min = 1, max = 32))]
    pub owner_id: String,

    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(min = 3, max = 3, message = "currency code must be a valid 3-letter ISO code"))]
    pub currency_code: String,

    #[serde(default)]
    pub is_neg_balance_allowed: bool,
}

#[derive(Debug, Serialize)]
pub struct LedgerAccountBalanceResp {
    pub currency_code: String,
    pub pending_balance: Decimal,
    pub available_balance: Decimal,
    pub posted_balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LedgerAccountBalance> for LedgerAccountBalanceResp {
    fn from(b: LedgerAccountBalance) -> Self {
        Self {
            currency_code: b.currency_code,
            pending_balance: b.pending_balance,
            available_balance: b.available_balance,
            posted_balance: b.posted_balance,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LedgerAccountResp {
    pub id: String,
    pub owner_id: String,
    pub name: Option<String>,
    pub is_neg_balance_allowed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub balances: Vec<LedgerAccountBalanceResp>,
}
