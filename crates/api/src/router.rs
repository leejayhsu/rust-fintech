use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_cookies::CookieManagerLayer;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::controllers;

pub fn build(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(controllers::health::check))
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

fn api_routes() -> Router<PgPool> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/users", user_routes())
        .nest("/ledger", ledger_routes())
        .nest("/parties", party_routes())
        .nest("/onboardings", onboarding_routes())
        .nest("/admin", admin_routes())
        .nest("/internal", internal_routes())
}

fn auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/signup", post(controllers::auth::signup))
        .route("/signin", post(controllers::auth::signin))
        .route("/logout", post(controllers::auth::logout))
        .route("/me", get(controllers::auth::me))
}

fn user_routes() -> Router<PgPool> {
    Router::new().route("/{id}", get(controllers::users::get))
}

fn ledger_routes() -> Router<PgPool> {
    Router::new().nest("/accounts", ledger_account_routes())
}

fn ledger_account_routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(controllers::ledger_accounts::create))
        .route("/{id}", get(controllers::ledger_accounts::get))
}

fn party_routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(controllers::parties::create))
        .route("/{id}", get(controllers::parties::get))
}

fn onboarding_routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/client",
            post(controllers::onboarding::create_client).get(controllers::onboarding::list_client),
        )
        .route("/client/{id}", get(controllers::onboarding::get_client))
}

fn admin_routes() -> Router<PgPool> {
    Router::new()
        .route("/onboardings", get(controllers::onboarding::list_admin))
        .route("/onboardings/{id}", get(controllers::onboarding::get_admin))
        .route(
            "/onboardings/{id}/review",
            post(controllers::onboarding::review_admin),
        )
}

fn internal_routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/onboardings/{id}/kyb-results",
            post(controllers::onboarding::record_kyb_results),
        )
        .route(
            "/onboardings/{id}/complete",
            post(controllers::onboarding::complete),
        )
}
