use axum::{
    extract::{rejection::JsonRejection, State},
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
                "40003",
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "40004",
            &format!("validation failed: {e}"),
        );
    }

    match ledger_service::create(&pool, body).await {
        Ok(account) => errors::success(account),
        Err(LedgerError::CurrencyNotFound) => errors::error(
            StatusCode::BAD_REQUEST,
            LedgerError::CurrencyNotFound.code(),
            &LedgerError::CurrencyNotFound.desc(),
        ),
        Err(LedgerError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = LedgerError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
    }
}
