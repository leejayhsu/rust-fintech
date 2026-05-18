#!/usr/bin/env bash
set -euo pipefail

DB_NAME="rust_fintech"
DB_USER="postgres"
DB_HOST="localhost"
DB_PORT="5432"
DB_PASSWORD="postgres"

export PGPASSWORD="${DB_PASSWORD}"

echo "Disconnecting all clients from ${DB_NAME}..."

# Connect to the 'postgres' maintenance DB to terminate other connections
psql \
  -h "${DB_HOST}" \
  -p "${DB_PORT}" \
  -U "${DB_USER}" \
  -d "postgres" \
  -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${DB_NAME}' AND pid <> pg_backend_pid();" \
  > /dev/null 2>&1 || true

echo "Dropping database ${DB_NAME}..."
psql \
  -h "${DB_HOST}" \
  -p "${DB_PORT}" \
  -U "${DB_USER}" \
  -d "postgres" \
  -c "DROP DATABASE IF EXISTS ${DB_NAME};"

echo "Creating database ${DB_NAME}..."
psql \
  -h "${DB_HOST}" \
  -p "${DB_PORT}" \
  -U "${DB_USER}" \
  -d "postgres" \
  -c "CREATE DATABASE ${DB_NAME};"

echo "Database ${DB_NAME} has been reset."
