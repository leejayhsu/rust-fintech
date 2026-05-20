# API

## Project Overview

Rust REST API in `crates/api` using Axum, SQLx, and thiserror. Follows a controller/service architecture with strict layer separation.

## Architecture

### Layer Responsibilities

**Controllers** (`crates/api/src/controllers/`)
- Parse and extract HTTP request data (path params, query params, request body)
- Validate request structure and format (see Validation section)
- Call one or more services
- Map service results (including errors) to appropriate HTTP responses
- Return `impl IntoResponse`

**Services** (`crates/api/src/services/`)
- Contain all business logic
- Interact with the database via SQLx
- Return `Result<T, SomeError>` using thiserror-defined error types
- Never reference HTTP types (`StatusCode`, `HeaderMap`, etc.)
- Services should be a single unit of work, like CreateUser, GetUser, DeleteUser, etc.

**Models** (`crates/api/src/models/`)
- For each entity, we should have various representations:
  - entity from serialized from DB
  - entity returned in api response (here is where we will ommit sensitive fields, and other things api clients dont need to know about). should follow {HTTPAction}{Entity}{Req|Resp}, generally, when a model is used for the HTTP layer.
  - colocate request with entity models.
  - A full list of models for a user entity might be:
    - CreateUserReq
    - UpdateUserReq
    - UserResp (any api that returns a user object would use this model)
    - User (DB row struct)

** DB layer**
- prefer text columns instead of varchar
- snake_case column names
- pluralized table names


**API layer**
- use a unified api response format
- example success response, non-paginated
```json
{
  "success": true,
  "error": null,
  "data": {}
}
```
- example success response, paginated
```json
{
  "success": true,
  "error": null,
  "data": {
    "items": [],
    "total": 0
  }
}
```
- example error response
```json
{
  "success": false,
  "error": {
    "code": "", // stable, machine readable error code
    "desc": "" // human readable error description
  },
  "data": null
}
```
- every error should have a numeric code. it should be 5 digits, and have a basic encoding.
- first digit = domain (auth, user, transactions, etc)
- every digit after that should just be incrementing from 0 for each error type.
- keep track of error codes in an enum.

examples:
auth errors
10001 - weak password
10002 - invalid credentials

user errors
20001 - user not found
20002 - update now allowed

transaction errors
30001 - insufficient funds
30002 - transaction not found

### File Layout

```
crates/api/
  src/
    controllers/
      mod.rs
      users.rs
    services/
      mod.rs
      users.rs
    errors/
      mod.rs
      user_error.rs   # one error enum per domain
    models/
      mod.rs
      user.rs         # DB row structs, request/response types
    db.rs             # pool setup
    main.rs
    router.rs
  migrations/
```

---

## Error Handling

### Service Errors

Define errors with `thiserror` in `src/errors/`. One enum per domain.

```rust
// src/errors/user_error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user not found")]
    NotFound,

    #[error("email already in use")]
    EmailConflict,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Controller Error Mapping

Controllers must handle **every** variant from a service error and return a typed JSON error response. Do not use `.unwrap()` or `?` at the controller boundary — match explicitly.

```rust
// src/controllers/users.rs
pub async fn get_user(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_service::find_by_id(&pool, user_id).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(UserError::NotFound) => error_response(StatusCode::NOT_FOUND, "user not found"),
        Err(UserError::Database(e)) => {
            tracing::error!("db error: {e}");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
        Err(e) => {
            tracing::error!("unhandled error: {e}");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
    }
}
```

Use a shared helper for JSON error responses:

```rust
// src/errors/mod.rs
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    (status, Json(json!({ "error": message }))).into_response()
}
```

### JSON Deserialization Errors

Axum's `Json` extractor returns a plain-text `422 Unprocessable Entity` by default when the body fails to deserialize. Controllers must catch this and map it to the unified API error format. Accept `Result<Json<T>, JsonRejection>` in the handler signature:

```rust
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn create_user(
    State(pool): State<PgPool>,
    body: Result<Json<CreateUserRequest>, JsonRejection>,
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

    // ... proceed with validation / service call
}
```

---

## Validation

### What Controllers Validate (HTTP concern)

- Required fields are present
- Field types and formats (UUID shape, email format, string length bounds)
- Enum/allowed-value constraints on input
- Query param parsing

Use `validator` crate with `#[derive(Validate)]` on request structs. Reject invalid input before calling any service.

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 2, max = 100))]
    pub name: String,
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({ "error": e }))).into_response();
    }

    match user_service::create(&pool, body).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(UserError::EmailConflict) => error_response(StatusCode::CONFLICT, "email already in use"),
        Err(e) => {
            tracing::error!("{e}");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
    }
}
```

### What Services Validate (domain concern)

- Business rule violations (uniqueness checks, ownership, state machine transitions)
- Cross-entity consistency
- Return specific error variants, never panic

---

## Database (SQLx)

- Use `PgPool` passed via Axum `State`
- Use `query_as!` macro with compile-time checking where possible
- Wrap SQLx errors in domain error variants via `#[from]` — do not leak `sqlx::Error` out of services
- for local development, you can assume the db is running at: DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech (useful for compile time type checking)

```rust
// src/services/users.rs
use sqlx::PgPool;
use crate::{errors::user_error::UserError, models::user::User};

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<User, UserError> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await?
        .ok_or(UserError::NotFound)
}
```

---

## Axum

### Dependencies

```toml
[dependencies]
axum = { version = "0.8", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors", "compression-gzip"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono"] }
thiserror = "2"
validator = { version = "0.19", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

### Router Setup

Define routes in `src/router.rs`, not in `main.rs`. Group routes by domain and nest them under a shared prefix.

```rust
// src/router.rs
use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use crate::controllers::{users, health};

pub fn build(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(health::check))
        .with_state(pool)
}

fn api_routes() -> Router<PgPool> {
    Router::new()
        .nest("/users", user_routes())
}

fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(users::create))
        .route("/:id", get(users::get).put(users::update).delete(users::delete))
}
```

### main.rs

Keep `main.rs` minimal — only wiring.

```rust
// src/main.rs
mod controllers;
mod db;
mod errors;
mod models;
mod router;
mod services;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let pool = db::connect().await;
    let app = router::build(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

### Middleware

Apply middleware in `router.rs` via `layer()`. Order matters — layers are applied bottom-up.

```rust
use tower_http::{cors::CorsLayer, trace::TraceLayer, compression::CompressionLayer};

pub fn build(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(health::check))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()) // tighten in production
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
```

### Extractors

Use Axum's built-in extractors in controller function signatures. Always put `State` first.

```rust
// order: State, then path/query, then body (body extractor must be last)
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUserRequest>,
) -> impl IntoResponse { ... }

pub async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse { ... }

pub async fn list_users(
    State(pool): State<PgPool>,
    Query(params): Query<ListUsersParams>,
) -> impl IntoResponse { ... }
```

### Shared App State

If you need more than just a DB pool in state (e.g. an S3 client, config, feature flags), define an `AppState` struct instead of passing `PgPool` directly.

```rust
// src/state.rs
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}
```

Update `Router<PgPool>` to `Router<AppState>` throughout and extract with `State(state): State<AppState>` in controllers.

---

## API Testing

When providing curl examples or test commands, **never include a trailing slash** on routes. Axum does not normalize trailing slashes by default, so a request to `/api/v1/users/` will 404 while `/api/v1/users` succeeds.

## General Rules

- No `unwrap()` or `expect()` in production paths — use `?` or explicit error handling
- Services never import from `axum::http` or touch HTTP types
- Controllers never contain SQL or business logic
- All errors must be logged at the controller boundary before returning 5xx responses
- Prefer explicit `match` over `map_err` chains when error variants need different HTTP status codes

# FE

## Project Overview

Frontend code lives under `apps/web` and shared frontend packages live under `packages/*`. Treat this as a TypeScript React frontend that talks to the Rust API over HTTP.

## Stack

- Use TypeScript for all frontend code.
- Use React for UI.
- Use shadcn/ui as the UI component library, with the Radix-backed components and local component source under `apps/web/src/components/ui`.
- Use TanStack Router for client-side routing.
- Use TanStack Form for forms and form validation flows.
- Use TanStack Query for server state, caching, mutations, invalidation, and request lifecycle handling.
- Default to TanStack libraries when there is a reasonable TanStack option.
- Do not use TanStack Start. This monorepo has a separate Rust backend, so the frontend should remain a standalone React app that calls the API.

## API Client And Type Safety

Use OpenAPI as the contract between the Rust API and the TypeScript frontend.

- Maintain the OpenAPI spec manually in `docs/openapi.yaml`.
- Treat `docs/openapi.yaml` as part of the API source of truth. When routes, request bodies, response bodies, status codes, or error shapes change, update the spec in the same change.
- Keep the manual spec aligned with the Axum controllers and Rust request/response models. Do not let implementation-only API changes drift from the spec.
- Generate a TypeScript API client from `docs/openapi.yaml` with Orval.
- Configure Orval to generate:
  - typed request and response interfaces
  - typed endpoint functions
  - TanStack Query hooks/options for queries and mutations
- Put generated client code in a shared package such as `packages/api-client` so `apps/web` can consume it without duplicating API types.
- Do not hand-write request/response interfaces in the frontend when they can be generated from the OpenAPI contract.
- Keep API route strings centralized in the generated client. Frontend feature code should call typed client functions/hooks instead of hardcoding fetch URLs.
- Regenerate the frontend client whenever API request/response models or routes change.
- Prefer small, reviewed OpenAPI edits over Rust-side OpenAPI derive/generation unless the manual spec becomes too expensive to maintain.

This gives the frontend an RPC-like developer experience while keeping the backend a conventional Rust HTTP API.

## Routing

- Define application routes with TanStack Router.
- Keep route definitions type-safe and colocate route-level data loading with the route where practical.
- Use generated API client calls inside TanStack Query loaders/hooks rather than calling `fetch` directly from components.

## Forms

- Use TanStack Form for complex or persisted forms.
- Prefer shared validation schemas/types from generated API contracts when possible.
- Keep UI validation aligned with backend validation, but treat backend validation as authoritative.

## Data Fetching

- Use TanStack Query for all server state.
- Prefer query keys that are stable, explicit, and colocated with the feature or generated client helpers.
- Use mutations for writes and invalidate or update relevant queries after successful mutations.
- Do not store server state in local React state except for transient UI interactions.
