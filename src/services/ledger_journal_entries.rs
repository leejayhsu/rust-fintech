use std::collections::HashSet;

use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::{
    errors::ledger_journal_entry_error::LedgerJournalEntryError,
    models::ledger_account::{LedgerAccount, LedgerAccountBalance},
    models::ledger_entry::LedgerEntry,
    models::ledger_journal_entry::{
        CreateExchangeLedgerJournalEntryReq, CreateLedgerJournalEntryLegReq, CreateLedgerJournalEntryReq, LedgerJournalEntry,
    },
    utils,
};

pub async fn create(
    pool: &PgPool,
    req: CreateLedgerJournalEntryReq,
) -> Result<(LedgerJournalEntry, Vec<LedgerEntry>), LedgerJournalEntryError> {
    validate_compound(&req)?;
    execute(pool, req.debits, req.credits, req.status).await
}

pub async fn create_exchange(
    pool: &PgPool,
    req: CreateExchangeLedgerJournalEntryReq,
) -> Result<(LedgerJournalEntry, Vec<LedgerEntry>), LedgerJournalEntryError> {
    validate_exchange(&req)?;
    let compound = CreateLedgerJournalEntryReq {
        debits: vec![req.debit],
        credits: vec![req.credit],
        status: req.status,
    };
    execute(pool, compound.debits, compound.credits, compound.status).await
}

async fn execute(
    pool: &PgPool,
    debits: Vec<CreateLedgerJournalEntryLegReq>,
    credits: Vec<CreateLedgerJournalEntryLegReq>,
    status: String,
) -> Result<(LedgerJournalEntry, Vec<LedgerEntry>), LedgerJournalEntryError> {
    let mut tx = pool.begin().await?;

    let all_account_ids: Vec<String> = debits
        .iter()
        .chain(credits.iter())
        .map(|l| l.account_id.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let found_accounts: Vec<String> = sqlx::query_scalar!(
        "SELECT id FROM ledger_accounts WHERE id = ANY($1)",
        &all_account_ids[..]
    )
    .fetch_all(&mut *tx)
    .await?;

    if found_accounts.len() != all_account_ids.len() {
        return Err(LedgerJournalEntryError::AccountNotFound);
    }

    let journal_id = utils::generate_id("lje");
    let now = chrono::Utc::now();

    let journal_entry = sqlx::query_as!(
        LedgerJournalEntry,
        r#"
        INSERT INTO ledger_journal_entries (id, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, status, created_at, updated_at
        "#,
        journal_id,
        status,
        now,
        now
    )
    .fetch_one(&mut *tx)
    .await?;

    let mut entries = Vec::with_capacity(debits.len() + credits.len());

    for leg in debits {
        let balance = sqlx::query_as!(
            LedgerAccountBalance,
            r#"
            SELECT account_id, currency_code, pending_balance, available_balance, posted_balance, created_at, updated_at
            FROM ledger_account_balances
            WHERE account_id = $1 AND currency_code = $2
            "#,
            leg.account_id,
            leg.currency_code
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(LedgerJournalEntryError::BalanceNotFound)?;

        let account = sqlx::query_as!(
            LedgerAccount,
            r#"
            SELECT id, owner_id, name, is_neg_balance_allowed, created_at, updated_at
            FROM ledger_accounts
            WHERE id = $1
            "#,
            leg.account_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if !account.is_neg_balance_allowed && balance.available_balance < leg.amount {
            return Err(LedgerJournalEntryError::InsufficientFunds);
        }

        let entry = sqlx::query_as!(
            LedgerEntry,
            r#"
            INSERT INTO ledger_entries (id, ledger_journal_entry_id, account_id, currency_code, amount, direction, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, ledger_journal_entry_id, account_id, currency_code, amount, direction, created_at, updated_at
            "#,
            utils::generate_id("lent"),
            journal_id,
            leg.account_id,
            leg.currency_code,
            leg.amount,
            "debit",
            now,
            now
        )
        .fetch_one(&mut *tx)
        .await?;

        entries.push(entry);

        match status.as_str() {
            "posted" => {
                sqlx::query!(
                    r#"
                    UPDATE ledger_account_balances
                    SET posted_balance = posted_balance - $1,
                        available_balance = available_balance - $1,
                        updated_at = NOW()
                    WHERE account_id = $2 AND currency_code = $3
                    "#,
                    leg.amount,
                    leg.account_id,
                    leg.currency_code
                )
                .execute(&mut *tx)
                .await?;
            }
            "pending" => {
                sqlx::query!(
                    r#"
                    UPDATE ledger_account_balances
                    SET available_balance = available_balance - $1,
                        pending_balance = pending_balance + $1,
                        updated_at = NOW()
                    WHERE account_id = $2 AND currency_code = $3
                    "#,
                    leg.amount,
                    leg.account_id,
                    leg.currency_code
                )
                .execute(&mut *tx)
                .await?;
            }
            _ => unreachable!(),
        }
    }

    for leg in credits {
        let balance_exists = sqlx::query_scalar!(
            "SELECT 1 FROM ledger_account_balances WHERE account_id = $1 AND currency_code = $2",
            leg.account_id,
            leg.currency_code
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();

        if !balance_exists {
            return Err(LedgerJournalEntryError::BalanceNotFound);
        }

        let entry = sqlx::query_as!(
            LedgerEntry,
            r#"
            INSERT INTO ledger_entries (id, ledger_journal_entry_id, account_id, currency_code, amount, direction, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, ledger_journal_entry_id, account_id, currency_code, amount, direction, created_at, updated_at
            "#,
            utils::generate_id("lent"),
            journal_id,
            leg.account_id,
            leg.currency_code,
            leg.amount,
            "credit",
            now,
            now
        )
        .fetch_one(&mut *tx)
        .await?;

        entries.push(entry);

        match status.as_str() {
            "posted" => {
                sqlx::query!(
                    r#"
                    UPDATE ledger_account_balances
                    SET posted_balance = posted_balance + $1,
                        available_balance = available_balance + $1,
                        updated_at = NOW()
                    WHERE account_id = $2 AND currency_code = $3
                    "#,
                    leg.amount,
                    leg.account_id,
                    leg.currency_code
                )
                .execute(&mut *tx)
                .await?;
            }
            "pending" => {
                sqlx::query!(
                    r#"
                    UPDATE ledger_account_balances
                    SET pending_balance = pending_balance + $1,
                        updated_at = NOW()
                    WHERE account_id = $2 AND currency_code = $3
                    "#,
                    leg.amount,
                    leg.account_id,
                    leg.currency_code
                )
                .execute(&mut *tx)
                .await?;
            }
            _ => unreachable!(),
        }
    }

    tx.commit().await?;

    Ok((journal_entry, entries))
}

fn validate_compound(req: &CreateLedgerJournalEntryReq) -> Result<(), LedgerJournalEntryError> {
    if req.debits.is_empty() || req.credits.is_empty() {
        return Err(LedgerJournalEntryError::InvalidJournalEntry);
    }

    if req.status != "pending" && req.status != "posted" {
        return Err(LedgerJournalEntryError::InvalidJournalEntry);
    }

    for leg in req.debits.iter().chain(req.credits.iter()) {
        if leg.amount <= Decimal::ZERO {
            return Err(LedgerJournalEntryError::InvalidJournalEntry);
        }
    }

    let all_currencies: HashSet<&str> = req
        .debits
        .iter()
        .chain(req.credits.iter())
        .map(|l| l.currency_code.as_str())
        .collect();

    if all_currencies.len() == 1 {
        let sum_debits: Decimal = req.debits.iter().map(|l| l.amount).sum();
        let sum_credits: Decimal = req.credits.iter().map(|l| l.amount).sum();
        if sum_debits != sum_credits {
            return Err(LedgerJournalEntryError::InvalidJournalEntry);
        }
    } else {
        if req.debits.len() != 1 || req.credits.len() != 1 {
            return Err(LedgerJournalEntryError::InvalidJournalEntry);
        }
    }

    Ok(())
}

fn validate_exchange(req: &CreateExchangeLedgerJournalEntryReq) -> Result<(), LedgerJournalEntryError> {
    if req.debit.currency_code == req.credit.currency_code {
        return Err(LedgerJournalEntryError::InvalidJournalEntry);
    }

    if req.debit.amount <= Decimal::ZERO || req.credit.amount <= Decimal::ZERO {
        return Err(LedgerJournalEntryError::InvalidJournalEntry);
    }

    if req.status != "pending" && req.status != "posted" {
        return Err(LedgerJournalEntryError::InvalidJournalEntry);
    }

    Ok(())
}
