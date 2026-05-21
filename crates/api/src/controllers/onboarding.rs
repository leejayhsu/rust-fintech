use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::{AdminUser, AuthUser},
    errors::{self, onboarding_error::OnboardingError},
    models::onboarding::{
        AdminReviewClientOnboardingReq, CreateClientOnboardingReq, ListClientOnboardingsQuery,
        RecordKybResultsReq, VALID_ONBOARDING_STATUSES,
    },
    services::{onboarding as onboarding_service, onboarding_worker_client as worker_client},
};

pub async fn create_client(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    body: Result<Json<CreateClientOnboardingReq>, JsonRejection>,
) -> impl IntoResponse {
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            let err = OnboardingError::InvalidRequest;
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                err.code(),
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        let err = OnboardingError::InvalidRequest;
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            err.code(),
            &format!("validation failed: {e}"),
        );
    }

    match onboarding_service::create(&pool, &auth_user.user.id, body).await {
        Ok(onboarding) => {
            let Some(workflow_id) = onboarding.temporal_workflow_id.as_deref() else {
                tracing::error!("created onboarding without workflow id: {}", onboarding.id);
                return map_onboarding_error(OnboardingError::TemporalStartFailed);
            };

            match worker_client::start_client_onboarding(&onboarding.id, workflow_id).await {
                Ok(()) => errors::success(onboarding),
                Err(e) => {
                    tracing::error!("failed to start onboarding workflow: {e}");
                    if let Err(mark_err) =
                        onboarding_service::set_failed(&pool, &onboarding.id).await
                    {
                        tracing::error!("failed to mark onboarding failed: {mark_err}");
                    }
                    map_onboarding_error(OnboardingError::TemporalStartFailed)
                }
            }
        }
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn list_client(State(pool): State<PgPool>, auth_user: AuthUser) -> impl IntoResponse {
    match onboarding_service::list_for_submitted_user(&pool, &auth_user.user.id).await {
        Ok(onboardings) => errors::success(onboardings),
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn get_client(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(onboarding_id): Path<String>,
) -> impl IntoResponse {
    match onboarding_service::get_for_submitted_user(&pool, &auth_user.user.id, &onboarding_id)
        .await
    {
        Ok(onboarding) => errors::success(onboarding),
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn list_admin(
    State(pool): State<PgPool>,
    _admin_user: AdminUser,
    Query(query): Query<ListClientOnboardingsQuery>,
) -> impl IntoResponse {
    if let Some(status) = query.status.as_deref() {
        if !VALID_ONBOARDING_STATUSES.contains(&status) {
            return map_onboarding_error(OnboardingError::InvalidStatus);
        }
    }

    match onboarding_service::list_for_admin(&pool, query.status.as_deref()).await {
        Ok(onboardings) => errors::success(onboardings),
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn get_admin(
    State(pool): State<PgPool>,
    _admin_user: AdminUser,
    Path(onboarding_id): Path<String>,
) -> impl IntoResponse {
    match onboarding_service::get_for_admin(&pool, &onboarding_id).await {
        Ok(onboarding) => errors::success(onboarding),
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn review_admin(
    State(pool): State<PgPool>,
    admin_user: AdminUser,
    Path(onboarding_id): Path<String>,
    body: Result<Json<AdminReviewClientOnboardingReq>, JsonRejection>,
) -> impl IntoResponse {
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            let err = OnboardingError::InvalidRequest;
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                err.code(),
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        let err = OnboardingError::InvalidRequest;
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            err.code(),
            &format!("validation failed: {e}"),
        );
    }

    match onboarding_service::record_admin_decision(
        &pool,
        &onboarding_id,
        &admin_user.user.id,
        AdminReviewClientOnboardingReq {
            approved: body.approved,
            note: body.note.clone(),
        },
    )
    .await
    {
        Ok(onboarding) => match worker_client::signal_admin_review(&onboarding_id, &body).await {
            Ok(()) => errors::success(onboarding),
            Err(e) => {
                // The worker bridge is expected to handle same-decision duplicate
                // review signals idempotently, so clients can retry this request
                // after a signal timeout without changing the stored decision.
                tracing::error!("failed to signal onboarding workflow review: {e}");
                map_onboarding_error(OnboardingError::TemporalSignalFailed)
            }
        },
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn record_kyb_results(
    State(pool): State<PgPool>,
    Path(onboarding_id): Path<String>,
    headers: HeaderMap,
    body: Result<Json<RecordKybResultsReq>, JsonRejection>,
) -> impl IntoResponse {
    if let Err(response) = authorize_internal(&headers) {
        return response;
    }

    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            let err = OnboardingError::InvalidRequest;
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                err.code(),
                &rejection.to_string(),
            );
        }
    };

    match onboarding_service::record_kyb_results(
        &pool,
        &onboarding_id,
        body.vendor_a,
        body.vendor_b,
    )
    .await
    {
        Ok(onboarding) => errors::success(onboarding),
        Err(e) => map_onboarding_error(e),
    }
}

pub async fn complete(
    State(pool): State<PgPool>,
    Path(onboarding_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = authorize_internal(&headers) {
        return response;
    }

    match onboarding_service::complete_with_originator_party(&pool, &onboarding_id).await {
        Ok(onboarding) => errors::success(onboarding),
        Err(e) => map_onboarding_error(e),
    }
}

fn authorize_internal(headers: &HeaderMap) -> Result<(), Response> {
    let Ok(token) = internal_api_token() else {
        tracing::error!("INTERNAL_API_TOKEN is not set or is blank");
        return Err(map_onboarding_error(OnboardingError::InternalUnauthorized));
    };

    let expected = format!("Bearer {token}");

    match headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        Some(actual) if actual == expected => Ok(()),
        _ => Err(map_onboarding_error(OnboardingError::InternalUnauthorized)),
    }
}

fn internal_api_token() -> Result<String, OnboardingError> {
    let token =
        std::env::var("INTERNAL_API_TOKEN").map_err(|_| OnboardingError::InternalUnauthorized)?;

    normalize_internal_api_token(&token)
}

fn normalize_internal_api_token(token: &str) -> Result<String, OnboardingError> {
    let token = token.trim().to_string();
    if token.is_empty() {
        Err(OnboardingError::InternalUnauthorized)
    } else {
        Ok(token)
    }
}

fn map_onboarding_error(err: OnboardingError) -> Response {
    match err {
        OnboardingError::NotFound => errors::error(StatusCode::NOT_FOUND, err.code(), &err.desc()),
        OnboardingError::InvalidStatus => {
            errors::error(StatusCode::CONFLICT, err.code(), &err.desc())
        }
        OnboardingError::TemporalStartFailed => {
            errors::error(StatusCode::BAD_GATEWAY, err.code(), &err.desc())
        }
        OnboardingError::TemporalSignalFailed => {
            errors::error(StatusCode::BAD_GATEWAY, err.code(), &err.desc())
        }
        OnboardingError::KybRejected => {
            errors::error(StatusCode::UNPROCESSABLE_ENTITY, err.code(), &err.desc())
        }
        OnboardingError::Database(e) => {
            tracing::error!("db error: {e}");
            let err = OnboardingError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        OnboardingError::InternalUnauthorized => {
            errors::error(StatusCode::UNAUTHORIZED, err.code(), &err.desc())
        }
        OnboardingError::InvalidRequest => {
            errors::error(StatusCode::UNPROCESSABLE_ENTITY, err.code(), &err.desc())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::onboarding_error::OnboardingError;

    #[test]
    fn normalize_token_rejects_whitespace_only_values() {
        assert!(matches!(
            super::normalize_internal_api_token("   \t\n"),
            Err(OnboardingError::InternalUnauthorized)
        ));
    }

    #[test]
    fn normalize_token_trims_valid_values() {
        let token = super::normalize_internal_api_token("  internal-token  ")
            .expect("token should be valid");

        assert_eq!(token, "internal-token");
    }
}
