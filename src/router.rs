use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::controllers;

pub fn build(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(controllers::health::check))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

fn api_routes() -> Router<PgPool> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/users", user_routes())
}

fn auth_routes() -> Router<PgPool> {
    Router::new().route("/signup", post(controllers::auth::signup))
}

fn user_routes() -> Router<PgPool> {
    Router::new().route("/{id}", get(controllers::users::get))
}
