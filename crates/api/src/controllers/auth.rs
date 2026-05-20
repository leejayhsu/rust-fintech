use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use tower_cookies::Cookies;
use validator::Validate;

use crate::{
    auth::{AuthUser, SESSION_COOKIE_NAME, build_session_cookie, build_session_removal_cookie},
    errors::{self, auth_error::AuthError, user_error::UserError},
    models::{auth::SigninReq, user::UserResp},
    services::{auth as auth_service, sessions as session_service, users as user_service},
};

pub async fn signup(
    State(pool): State<PgPool>,
    body: Result<Json<crate::models::user::CreateUserReq>, JsonRejection>,
) -> impl IntoResponse {
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "20006",
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "20005",
            &format!("validation failed: {e}"),
        );
    }

    match user_service::create(&pool, body).await {
        Ok(user) => errors::success(UserResp::from(user)),
        Err(UserError::EmailConflict) => errors::error(
            StatusCode::CONFLICT,
            UserError::EmailConflict.code(),
            &UserError::EmailConflict.desc(),
        ),
        Err(UserError::PasswordHash) => {
            tracing::error!("password hashing failed");
            let err = UserError::PasswordHash;
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(UserError::Database(e)) => {
            tracing::error!("db error: {e}");
            let err = UserError::Database(e);
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, err.code(), &err.desc())
        }
        Err(e) => {
            tracing::error!("unhandled error: {e}");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, e.code(), &e.desc())
        }
    }
}

pub async fn signin(
    State(pool): State<PgPool>,
    cookies: Cookies,
    body: Result<Json<SigninReq>, JsonRejection>,
) -> impl IntoResponse {
    let Json(body) = match body {
        Ok(body) => body,
        Err(rejection) => {
            return errors::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "10005",
                &rejection.to_string(),
            );
        }
    };

    if let Err(e) = body.validate() {
        return errors::error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "10006",
            &format!("validation failed: {e}"),
        );
    }

    match auth_service::verify_credentials(&pool, &body.email, &body.password).await {
        Ok(user) => match session_service::create(&pool, &user.id).await {
            Ok(token) => {
                cookies.add(build_session_cookie(&token));
                errors::success(UserResp::from(user))
            }
            Err(e) => {
                tracing::error!("session creation failed: {e}");
                errors::error(StatusCode::INTERNAL_SERVER_ERROR, e.code(), &e.desc())
            }
        },
        Err(AuthError::InvalidCredentials) => errors::error(
            StatusCode::UNAUTHORIZED,
            AuthError::InvalidCredentials.code(),
            &AuthError::InvalidCredentials.desc(),
        ),
        Err(e) => {
            tracing::error!("signin error: {e}");
            errors::error(StatusCode::INTERNAL_SERVER_ERROR, e.code(), &e.desc())
        }
    }
}

pub async fn logout(State(pool): State<PgPool>, cookies: Cookies) -> impl IntoResponse {
    if let Some(cookie) = cookies.get(SESSION_COOKIE_NAME) {
        let token = cookie.value().to_string();
        if let Err(e) = session_service::delete_by_token(&pool, &token).await {
            tracing::error!("session deletion failed: {e}");
        }
    }

    cookies.remove(build_session_removal_cookie());
    errors::success(serde_json::json!({ "message": "logged out" }))
}

pub async fn me(auth_user: AuthUser) -> impl IntoResponse {
    errors::success(auth_user.user)
}
