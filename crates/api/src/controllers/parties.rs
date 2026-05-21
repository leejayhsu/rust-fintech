use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    errors::{self, party_error::PartyError},
    models::party::{CreatePartyReq, ORIGINATOR_PARTY_ROLE, VALID_PARTY_ROLES, VALID_PARTY_TYPES},
    services::parties as party_service,
};

pub async fn create(
    State(pool): State<PgPool>,
    body: Result<Json<CreatePartyReq>, JsonRejection>,
) -> impl IntoResponse {
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "50004",
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "50005",
            &format!("validation failed: {e}"),
        );
    }

    if !VALID_PARTY_TYPES.contains(&body.r#type.as_str()) {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "50006",
            "type must be either 'individual' or 'business'",
        );
    }

    if let Some(role) = body.role.as_deref() {
        if !VALID_PARTY_ROLES.contains(&role) {
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "50007",
                "role must be 'originator', 'beneficiary', or 'counterparty'",
            );
        }

        if role == ORIGINATOR_PARTY_ROLE && body.r#type != "business" {
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "50008",
                "originator parties must have type 'business'",
            );
        }
    }

    match party_service::create(&pool, body).await {
        Ok(party) => errors::success(party),
        Err(PartyError::PartyAlreadyExists) => errors::error(
            StatusCode::CONFLICT,
            PartyError::PartyAlreadyExists.code(),
            &PartyError::PartyAlreadyExists.desc(),
        ),
        Err(PartyError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = PartyError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(PartyError::PartyNotFound) => {
            tracing::error!("unexpected PartyNotFound in create");
            errors::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "50003",
                "internal server error",
            )
        }
    }
}

pub async fn get(State(pool): State<PgPool>, Path(party_id): Path<String>) -> impl IntoResponse {
    match party_service::find_by_id(&pool, &party_id).await {
        Ok(party) => errors::success(party),
        Err(PartyError::PartyNotFound) => errors::error(
            StatusCode::NOT_FOUND,
            PartyError::PartyNotFound.code(),
            &PartyError::PartyNotFound.desc(),
        ),
        Err(PartyError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = PartyError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(PartyError::PartyAlreadyExists) => {
            tracing::error!("unexpected PartyAlreadyExists in get");
            errors::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "50003",
                "internal server error",
            )
        }
    }
}
