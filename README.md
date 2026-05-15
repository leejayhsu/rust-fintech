# Rust Fintech API

A REST API built with Rust, Axum, SQLx, and PostgreSQL.

## Quickstart

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [PostgreSQL](https://www.postgresql.org/)

### Install dependencies

```bash
cargo build
```

### Set up environment

```bash
cp .env.example .env
# Edit .env with your database credentials
```

### Start the dev server

```bash
cargo run
```

The server will start on `http://localhost:3000`.

### Test the API

```bash
# Health check
curl http://localhost:3000/health

# Create a user
curl -X POST http://localhost:3000/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"secret123"}'
```

## Database Migrations

This project uses SQLx migrations. Migrations run automatically when the server starts.

### Create a migration

```bash
# Install sqlx-cli (one time)
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Create a new migration
sqlx migrate add <migration_name>
```

For example:

```bash
sqlx migrate add add_accounts_table
```

This creates a new timestamped `.sql` file in the `migrations/` directory.

### Run migrations

Migrations run automatically when you start the server (`cargo run`).

To run them manually:

```bash
export DATABASE_URL="postgres://user:password@localhost/dbname"
sqlx migrate run
```

### Check migration status

```bash
sqlx migrate info
```

### Revert a migration

```bash
sqlx migrate revert
```

## Project Structure

```
src/
  controllers/    # HTTP request handlers
  services/       # Business logic
  models/         # Request/response types and DB row structs
  errors/         # Domain error types
  db.rs           # Database pool setup
  router.rs       # Axum route definitions
  main.rs         # Application entry point

migrations/       # SQLx database migrations
```
