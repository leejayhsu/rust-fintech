use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::{
    errors, errors::auth_error::AuthError, models::user::UserResp,
    services::sessions as session_service,
};

pub const SESSION_COOKIE_NAME: &str = "session";

#[derive(Clone)]
pub struct AuthUser {
    pub user: UserResp,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AdminUser {
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

impl<S> FromRequestParts<S> for AdminUser
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        if auth_user.user.role != "admin" {
            let err = AuthError::AdminRequired;
            return Err(errors::error(
                StatusCode::FORBIDDEN,
                err.code(),
                &err.desc(),
            ));
        }

        Ok(AdminUser {
            user: auth_user.user,
        })
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

#[cfg(test)]
mod auth_roles {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        response::IntoResponse,
        routing::get,
    };
    use serde_json::Value;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use tower_cookies::CookieManagerLayer;

    use super::{AdminUser, SESSION_COOKIE_NAME};
    use crate::{
        errors,
        models::user::CreateUserReq,
        services::{sessions as session_service, users as user_service},
    };

    async fn admin_only(_admin: AdminUser) -> impl IntoResponse {
        errors::success(serde_json::json!({ "ok": true }))
    }

    async fn test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/rust_fintech".into());
        let pool = PgPool::connect(&database_url)
            .await
            .expect("test database should be available");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("test database migrations should apply");

        pool
    }

    async fn create_test_user(pool: &PgPool) -> (String, String) {
        let email = format!("auth_roles_{}@example.com", nanoid::nanoid!(12));
        let user = user_service::create(
            pool,
            CreateUserReq {
                email,
                password: "password123".to_string(),
            },
        )
        .await
        .expect("test user should be created");
        let token = session_service::create(pool, &user.id)
            .await
            .expect("test session should be created");

        (user.id, token)
    }

    #[tokio::test]
    async fn auth_roles_session_validation_includes_user_role() {
        let pool = test_pool().await;
        let (_user_id, token) = create_test_user(&pool).await;

        let user = session_service::validate_session(&pool, &token)
            .await
            .expect("test session should validate");

        assert_eq!(user.role, "user");
    }

    #[tokio::test]
    async fn auth_roles_admin_extractor_rejects_normal_users() {
        let pool = test_pool().await;
        let (_user_id, token) = create_test_user(&pool).await;
        let app = Router::new()
            .route("/admin-only", get(admin_only))
            .layer(CookieManagerLayer::new())
            .with_state(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin-only")
                    .header(header::COOKIE, format!("{SESSION_COOKIE_NAME}={token}"))
                    .body(Body::empty())
                    .expect("test request should build"),
            )
            .await
            .expect("test request should complete");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("test response body should be readable");
        let body: Value =
            serde_json::from_slice(&body).expect("test response body should be valid json");

        assert_eq!(body["success"], false);
        assert_eq!(body["error"]["code"], "10007");
    }
}
