use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::{errors, models::user::UserResp, services::sessions as session_service};

pub const SESSION_COOKIE_NAME: &str = "session";

#[derive(Clone)]
pub struct AuthUser {
    pub user: UserResp,
}

impl<S> FromRequestParts<S> for AuthUser
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|_| unreachable!("CookieManagerLayer is required"));

        let pool = PgPool::from_ref(state);

        let token = cookies
            .get(SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string())
            .ok_or_else(|| errors::error(StatusCode::UNAUTHORIZED, "10001", "not authenticated"))?;

        let user = session_service::validate_session(&pool, &token)
            .await
            .map_err(|e| match e {
                crate::errors::auth_error::AuthError::NotAuthenticated => {
                    errors::error(StatusCode::UNAUTHORIZED, e.code(), &e.desc())
                }
                _ => {
                    tracing::error!("session validation error: {e}");
                    errors::error(StatusCode::INTERNAL_SERVER_ERROR, e.code(), &e.desc())
                }
            })?;

        Ok(AuthUser { user })
    }
}

pub fn build_session_cookie(token: &str) -> tower_cookies::Cookie<'static> {
    let mut cookie = tower_cookies::Cookie::new(SESSION_COOKIE_NAME, token.to_string());
    cookie.set_http_only(true);
    cookie.set_secure(is_cookie_secure());
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_path("/");
    cookie
}

pub fn build_session_removal_cookie() -> tower_cookies::Cookie<'static> {
    let mut cookie = tower_cookies::Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_http_only(true);
    cookie.set_secure(is_cookie_secure());
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::seconds(0));
    cookie
}

fn is_cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE")
        .map(|v| v != "false")
        .unwrap_or(true)
}
