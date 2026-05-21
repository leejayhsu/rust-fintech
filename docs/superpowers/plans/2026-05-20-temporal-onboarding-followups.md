# Temporal Onboarding Follow-Ups Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the completed Temporal client onboarding implementation after code review, while preserving the live E2E behavior already verified.

**Architecture:** The core flow already works: user onboarding submission starts Temporal, two fake KYB vendors run concurrently, admin approval signals the workflow, and approval creates an originator business party. This plan addresses security, retry correctness, contract completeness, dependency declaration, and regression tests.

**Tech Stack:** Rust, Axum, SQLx, Temporal TypeScript SDK, Express bridge, React, OpenAPI, Orval.

---

## Current Verified State

E2E smoke verification succeeded:

- Submitted onboarding `onb_IdvtYam6A8dWrj`.
- KYB moved from `kyb_pending` to `manual_review_pending`.
- Admin approval moved status to `approved`.
- Created party `party_oF94XjCiKkAoF8`.
- Created party has `type = 'business'` and `role = 'originator'`.

The remaining work is hardening and cleanup, not proving the happy path.

## Task 1: Secure API-To-Worker Bridge

**Files:**
- Modify: `crates/api/src/services/onboarding_worker_client.rs`
- Modify: `apps/temporal-worker/src/client.ts`
- Modify: `docs/onboarding-temporal-dev.md`

- [ ] Require `ONBOARDING_WORKER_BRIDGE_TOKEN` in the Rust worker client.
- [ ] Send `Authorization: Bearer ${ONBOARDING_WORKER_BRIDGE_TOKEN}` on bridge start/review calls.
- [ ] Require the same token in the Express bridge before accepting `/internal/workflows/client-onboarding/start` or `/review`.
- [ ] Reject missing, empty, or whitespace-only bridge tokens.
- [ ] Bind the Express bridge to `127.0.0.1` by default.
- [ ] Add `ONBOARDING_WORKER_HOST` only if explicit non-loopback binding is needed.
- [ ] Update local dev docs with the new bridge token env var.

Verification:

```bash
npm run worker:check
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api
```

## Task 2: Tighten Review Signal Idempotency

**Files:**
- Modify: `apps/temporal-worker/src/client.ts`
- Modify: `crates/api/src/services/onboarding_worker_client.rs` if contract comments change
- Optional modify: `crates/api/src/services/onboarding.rs`

- [ ] Stop treating broad `not found`, `closed`, `completed`, `terminated`, or `canceled` Temporal errors as success.
- [ ] Return success for duplicate same-decision review signals only when the bridge can safely prove the same workflow decision was already accepted or the API state is terminal and consistent.
- [ ] If safe proof is awkward in the bridge, add a small durable signal-delivery marker/outbox in the Rust API instead of relying on error text.
- [ ] Keep retry behavior for API timeouts: a same approval retry should be able to signal again without changing the DB decision.

Verification:

```bash
npm run worker:check
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api
```

## Task 3: Complete OpenAPI Error Surface

**Files:**
- Modify: `docs/openapi.yaml`
- Regenerate: `packages/api-client/src/generated/**`

- [ ] Add non-200 responses for onboarding/admin routes:
  - `401`
  - `403`
  - `404`
  - `409`
  - `422`
  - `502`
- [ ] Use the existing unified error envelope shape.
- [ ] Re-run Orval.

Verification:

```bash
npm run api-client:generate
npm run api-client:check
```

## Task 4: Declare Web API Client Dependency

**Files:**
- Modify: `apps/web/package.json`
- Modify: `package-lock.json`

- [ ] Add `@rust-fintech/api-client` as a dependency of `@rust-fintech/web`.
- [ ] Run `npm install`.

Verification:

```bash
npm --workspace @rust-fintech/web run typecheck
npm run web:check
```

## Task 5: Add Targeted Regression Tests

**Files:**
- Add or modify tests near the affected Rust and TypeScript modules.

- [ ] Add worker bridge auth tests.
- [ ] Add worker bridge review-signal tests for duplicate decision and missing/closed workflow behavior.
- [ ] Add Rust onboarding state-machine test if practical:
  - create onboarding
  - record KYB results
  - record admin approval
  - complete onboarding
  - assert party is `business` / `originator`
- [ ] Add an OpenAPI contract test if practical to ensure onboarding routes document expected error responses.

Verification:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api
npm run worker:check
npm run api-client:check
npm run web:check
```

## Final Verification

Run:

```bash
cargo fmt --check
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo check -p api
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api
npm run api-client:generate
npm run api-client:check
npm run worker:check
npm run web:check
npm --workspace @rust-fintech/web run typecheck
```

Repeat the full live E2E smoke only if bridge auth or signal-idempotency changes affect the smoke path.
