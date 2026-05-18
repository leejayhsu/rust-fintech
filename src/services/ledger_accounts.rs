use sqlx::PgPool;

use crate::{
    errors::ledger_error::LedgerError,
    models::ledger_account::{
        CreateLedgerAccountReq, LedgerAccount, LedgerAccountBalance, LedgerAccountResp,
    },
    utils,
};

pub async fn create(
    pool: &PgPool,
    req: CreateLedgerAccountReq,
) -> Result<LedgerAccountResp, LedgerError> {
    let user_exists = sqlx::query_scalar!(
        "SELECT 1 FROM users WHERE id = $1",
        req.owner_id
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if !user_exists {
        return Err(LedgerError::UserNotFound);
    }

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

    let mut tx = pool.begin().await?;

    let account = sqlx::query_as!(
        LedgerAccount,
        r#"
        INSERT INTO ledger_accounts (id, owner_id, name, is_neg_balance_allowed, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, owner_id, name, is_neg_balance_allowed, created_at, updated_at
        "#,
        utils::generate_id("lacc"),
        req.owner_id,
        req.name,
        req.is_neg_balance_allowed,
        chrono::Utc::now(),
        chrono::Utc::now()
    )
    .fetch_one(&mut *tx)
    .await?;

    let balance = sqlx::query_as!(
        LedgerAccountBalance,
        r#"
        INSERT INTO ledger_account_balances (account_id, currency_code, pending_balance, available_balance, posted_balance, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING account_id, currency_code, pending_balance, available_balance, posted_balance, created_at, updated_at
        "#,
        account.id,
        req.currency_code,
        rust_decimal::Decimal::ZERO,
        rust_decimal::Decimal::ZERO,
        rust_decimal::Decimal::ZERO,
        chrono::Utc::now(),
        chrono::Utc::now()
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            LedgerError::DuplicateCurrencyBalance
        }
        _ => LedgerError::Database(e),
    })?;

    tx.commit().await?;

    Ok(LedgerAccountResp {
        id: account.id,
        owner_id: account.owner_id,
        name: account.name,
        is_neg_balance_allowed: account.is_neg_balance_allowed,
        created_at: account.created_at,
        updated_at: account.updated_at,
        balances: vec![balance.into()],
    })
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<LedgerAccountResp, LedgerError> {
    let account = sqlx::query_as!(
        LedgerAccount,
        r#"
        SELECT id, owner_id, name, is_neg_balance_allowed, created_at, updated_at
        FROM ledger_accounts
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(LedgerError::AccountNotFound)?;

    let balances = sqlx::query_as!(
        LedgerAccountBalance,
        r#"
        SELECT account_id, currency_code, pending_balance, available_balance, posted_balance, created_at, updated_at
        FROM ledger_account_balances
        WHERE account_id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

    Ok(LedgerAccountResp {
        id: account.id,
        owner_id: account.owner_id,
        name: account.name,
        is_neg_balance_allowed: account.is_neg_balance_allowed,
        created_at: account.created_at,
        updated_at: account.updated_at,
        balances: balances.into_iter().map(|b| b.into()).collect(),
    })
}
