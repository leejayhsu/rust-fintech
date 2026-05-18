use sqlx::PgPool;

use crate::{
    errors::ledger_error::LedgerError,
    models::ledger_account::{CreateLedgerAccountReq, LedgerAccount, LedgerAccountResp},
    utils,
};

pub async fn create(pool: &PgPool, req: CreateLedgerAccountReq) -> Result<LedgerAccountResp, LedgerError> {
    let currency_exists = sqlx::query_scalar!(
        "SELECT 1 FROM currencies WHERE code = $1",
        req.currency_code
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if !currency_exists {
        return Err(LedgerError::CurrencyNotFound);
    }

    let account = sqlx::query_as!(
        LedgerAccount,
        r#"
        INSERT INTO ledger_accounts (id, is_neg_balance_allowed, currency_code, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, pending_balance, available_balance, posted_balance, is_neg_balance_allowed, currency_code, created_at, updated_at
        "#,
        utils::generate_id("lacc"),
        req.is_neg_balance_allowed,
        req.currency_code,
        chrono::Utc::now(),
        chrono::Utc::now()
    )
    .fetch_one(pool)
    .await?;

    Ok(account.into())
}
