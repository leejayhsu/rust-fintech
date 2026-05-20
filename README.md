# Rust Fintech

A monorepo for a Rust fintech API and TypeScript frontend packages.

## Quickstart

Run commands from the repository root unless a step says otherwise.

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [PostgreSQL](https://www.postgresql.org/)
- [Node.js](https://nodejs.org/) for the frontend workspace packages

### Install dependencies

```bash
# Install JavaScript workspace dependencies
npm install

# Fetch and compile Rust dependencies
cargo build -p api
```

### Set up environment

```bash
cp .env.example .env
# Edit .env with your database credentials
```

The default local database URL is:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech
```

### Run the dev servers

Start the backend API from one terminal:

```bash
npm run api:dev
```

This runs `cargo run -p api` and serves the API on `http://localhost:3000`.

Start the frontend from another terminal:

```bash
npm run web:dev
```

This runs the `@rust-fintech/web` workspace dev script. The frontend is currently scaffolded, so the command prints a placeholder message until a web app is added.

You can also run the workspace commands directly:

```bash
cargo run -p api
npm --workspace @rust-fintech/web run dev
```

### Test the API

```bash
# Health check
curl http://localhost:3000/health

# Create a user
curl -X POST http://localhost:3000/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"secret123"}'
```

## API Contract

The API contract is maintained manually in `docs/openapi.yaml`.

When backend routes or request/response shapes change, update the OpenAPI spec in the same change. The frontend should generate its typed API client from this spec, so keeping it current is what gives the React app typed request/response interfaces and TanStack Query helpers.

## Database Migrations

This project uses SQLx migrations. Migrations run automatically when the server starts.

### Create a migration

```bash
# Install sqlx-cli (one time)
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Create a new migration
sqlx migrate add --source crates/api/migrations <migration_name>
```

For example:

```bash
sqlx migrate add --source crates/api/migrations add_accounts_table
```

This creates a new timestamped `.sql` file in the `crates/api/migrations/` directory.

### Run migrations

Migrations run automatically when you start the server (`cargo run -p api`).

To run them manually:

```bash
export DATABASE_URL="postgres://user:password@localhost/dbname"
sqlx migrate run --source crates/api/migrations
```

### Check migration status

```bash
sqlx migrate info --source crates/api/migrations
```

### Revert a migration

```bash
sqlx migrate revert --source crates/api/migrations
```

## Project Structure

```
Cargo.toml                 # Cargo workspace root
package.json               # npm workspace root
pnpm-workspace.yaml        # pnpm workspace root

crates/
  api/
    src/
      controllers/         # HTTP request handlers
      services/            # Business logic
      models/              # Request/response types and DB row structs
      errors/              # Domain error types
      db.rs                # Database pool setup
      router.rs            # Axum route definitions
      main.rs              # Application entry point
    migrations/            # SQLx database migrations

apps/
  web/                     # Future TypeScript React frontend

packages/
  shared-types/            # Future shared TypeScript types
```
