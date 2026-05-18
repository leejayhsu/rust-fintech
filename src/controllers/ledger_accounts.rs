use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    errors::{self, ledger_error::LedgerError},
    models::ledger_account::CreateLedgerAccountReq,
    services::ledger_accounts as ledger_service,
};

pub async fn create(
    State(pool): State<PgPool>,
    body: Result<Json<CreateLedgerAccountReq>, JsonRejection>,
) -> impl IntoResponse {
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "40006",
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "40007",
            &format!("validation failed: {e}"),
        );
    }

    match ledger_service::create(&pool, body).await {
        Ok(account) => errors::success(account),
        Err(LedgerError::UserNotFound) => errors::error(
            StatusCode::BAD_REQUEST,
            LedgerError::UserNotFound.code(),
            &LedgerError::UserNotFound.desc(),
        ),
        Err(LedgerError::CurrencyNotFound) => errors::error(
            StatusCode::BAD_REQUEST,
            LedgerError::CurrencyNotFound.code(),
            &LedgerError::CurrencyNotFound.desc(),
        ),
        Err(LedgerError::DuplicateCurrencyBalance) => errors::error(
            StatusCode::CONFLICT,
            LedgerError::DuplicateCurrencyBalance.code(),
            &LedgerError::DuplicateCurrencyBalance.desc(),
        ),
        Err(LedgerError::AccountNotFound) => errors::error(
            StatusCode::NOT_FOUND,
            LedgerError::AccountNotFound.code(),
            &LedgerError::AccountNotFound.desc(),
        ),
        Err(LedgerError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = LedgerError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
    }
}

pub async fn get(
    State(pool): State<PgPool>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    match ledger_service::find_by_id(&pool, &account_id).await {
        Ok(account) => errors::success(account),
        Err(LedgerError::AccountNotFound) => errors::error(
            StatusCode::NOT_FOUND,
            LedgerError::AccountNotFound.code(),
            &LedgerError::AccountNotFound.desc(),
        ),
        Err(LedgerError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = LedgerError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(LedgerError::UserNotFound) => {
            tracing::error!("unexpected UserNotFound in get");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, "40005", "internal server error")
        }
        Err(LedgerError::CurrencyNotFound) => {
            tracing::error!("unexpected CurrencyNotFound in get");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, "40005", "internal server error")
        }
        Err(LedgerError::DuplicateCurrencyBalance) => {
            tracing::error!("unexpected DuplicateCurrencyBalance in get");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, "40005", "internal server error")
        }
    }
}
