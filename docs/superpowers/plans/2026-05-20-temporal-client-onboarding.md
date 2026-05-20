# Temporal Client Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a durable client onboarding flow where business users submit company details, Temporal runs concurrent fake KYB checks, backoffice admins manually approve, and approval creates a `party` entity with role `originator`.

**Architecture:** Keep the Rust Axum API as the system of record and HTTP boundary. Add a TypeScript Temporal worker package because Temporal’s Rust crate is currently alpha-stage and documents workflow APIs as unstable, while Temporal’s official SDK chooser lists TypeScript among the primary SDKs. The Rust API will own users, admin authorization, onboarding records, and party creation; the Temporal worker will orchestrate fake KYB activities and wait for an approval signal initiated by the Rust admin API.

**Tech Stack:** Rust, Axum, SQLx, Postgres, thiserror, validator, Temporal TypeScript SDK, React, TanStack Router/Form/Query, shadcn/ui, OpenAPI + Orval.

---

## Design Decisions

- `parties.type` remains a legal party shape: `individual | business`. The requested “type originator” is modeled as `parties.role = 'originator'`, because originator describes how the party is used in money movement, not whether the entity is an individual or business.
- Admin users are first-class users with a role. Add `users.role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user', 'admin'))`.
- Development seed makes `leejayhsu@gmail.com` an admin without preventing that same account from being a normal app user.
- Onboarding is modeled as an auditable state machine in Postgres. Temporal is the durable orchestrator, but the API remains queryable even if Temporal UI is unavailable.
- Fake KYB vendors are activities, not workflow code, because they simulate non-deterministic external calls and timers.
- Manual approval is a Temporal signal. The admin API writes the approval decision transactionally, then signals the workflow.
- The final party is created only after both fake vendors pass and an admin approves.
- Use task queue `client-onboarding`. Use workflow id `client-onboarding-{onboarding_id}` for idempotency.

## Subagent Strategy

Use independent subagents with disjoint ownership. Workers are not alone in the codebase and must not revert edits made by others.

- **Subagent A: Backend schema and auth roles**
  Owns migrations, user role models, admin extractor, and auth response updates.
- **Subagent B: Backend onboarding API and party creation service**
  Owns onboarding models/errors/services/controllers/router and party role support.
- **Subagent C: Temporal worker package**
  Owns `apps/temporal-worker`, workflow, activities, worker bootstrap, and local scripts.
- **Subagent D: OpenAPI and generated client**
  Owns `docs/openapi.yaml`, Orval config, `packages/api-client`, and frontend API imports.
- **Subagent E: User onboarding wizard**
  Owns user-facing React route/components/styles for multi-step onboarding.
- **Subagent F: Admin portal**
  Owns admin React routes/components/styles for review and approve/reject.
- **Subagent G: Integration verification**
  Owns end-to-end dev verification notes, smoke scripts, and final cross-agent fixes only after all implementation workers finish.

## File Map

- Create `crates/api/migrations/20260520000001_add_user_roles_and_party_roles.sql`: add user roles, seed admin, add party role.
- Create `crates/api/migrations/20260520000002_create_client_onboardings.sql`: onboarding workflow state, KYB vendor results, and approval audit columns.
- Modify `crates/api/src/models/user.rs`: expose `role` on `User` and `UserResp`.
- Modify `crates/api/src/auth.rs`: add `AdminUser` extractor.
- Modify `crates/api/src/models/party.rs`: add `role`, validate `business` + `originator`.
- Modify `crates/api/src/services/parties.rs`: insert/select `role`.
- Create `crates/api/src/models/onboarding.rs`: request/response/status structs.
- Create `crates/api/src/errors/onboarding_error.rs`: `600xx` onboarding errors.
- Create `crates/api/src/services/onboarding.rs`: create/list/get/admin decision/complete methods.
- Create `crates/api/src/controllers/onboarding.rs`: user and admin endpoints.
- Modify `crates/api/src/router.rs`, `models/mod.rs`, `services/mod.rs`, `controllers/mod.rs`, `errors/mod.rs`.
- Modify `crates/api/Cargo.toml`: add Temporal client dependency only if Rust signaling is done directly; otherwise add `reqwest` for internal worker bridge.
- Create `apps/temporal-worker/package.json`, `tsconfig.json`, `src/workflows/client-onboarding.ts`, `src/activities/kyb.ts`, `src/activities/api.ts`, `src/worker.ts`, `src/client.ts`.
- Create or update `docs/openapi.yaml`, `orval.config.ts`, `packages/api-client/*`.
- Modify `package.json`: add `worker:dev`, `temporal:dev`, `api-client:generate`.
- Modify `apps/web/src/router.tsx`: add `/onboarding/client` and `/admin/onboarding`.
- Create `apps/web/src/routes/client-onboarding.tsx`.
- Create `apps/web/src/routes/admin-onboarding.tsx`.
- Modify `apps/web/src/styles.css`: app console, wizard, and admin queue styles.

---

### Task 1: Backend Schema And Roles

**Owner:** Subagent A

**Files:**
- Create: `crates/api/migrations/20260520000001_add_user_roles_and_party_roles.sql`
- Modify: `crates/api/src/models/user.rs`
- Modify: `crates/api/src/auth.rs`
- Modify: `crates/api/src/services/auth.rs`
- Modify: `crates/api/src/services/users.rs`
- Modify: `crates/api/src/controllers/auth.rs`

- [ ] **Step 1: Add failing role expectations**

Add or update API tests if a test harness exists. If no Rust API tests exist yet, create `crates/api/tests/auth_roles.rs` with tests for:

```rust
#[tokio::test]
async fn me_returns_user_role() {
    // Create user, sign in, call /api/v1/auth/me.
    // Assert response.data.role == "user".
}

#[tokio::test]
async fn admin_extractor_rejects_non_admin() {
    // Create normal user, sign in, call an admin-only route once Task 4 exists.
    // Assert 403 with error code 10007.
}
```

Run: `DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api auth_roles`

Expected: fail because `role` does not exist.

- [ ] **Step 2: Add migration**

Create:

```sql
ALTER TABLE users
ADD COLUMN role TEXT NOT NULL DEFAULT 'user'
CHECK (role IN ('user', 'admin'));

UPDATE users
SET role = 'admin'
WHERE email = 'leejayhsu@gmail.com';

ALTER TABLE parties
ADD COLUMN role TEXT NOT NULL DEFAULT 'counterparty'
CHECK (role IN ('originator', 'beneficiary', 'counterparty'));

CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_parties_role ON parties(role);
```

- [ ] **Step 3: Update user models**

Change `User` and `UserResp` to include:

```rust
pub role: String,
```

Update every `SELECT` and `RETURNING` for users to include `role`.

- [ ] **Step 4: Add admin extractor**

In `crates/api/src/auth.rs`, add:

```rust
#[derive(Clone)]
pub struct AdminUser {
    pub user: UserResp,
}
```

Implement `FromRequestParts` by delegating to `AuthUser`, then return:

```rust
if auth_user.user.role != "admin" {
    return Err(errors::error(StatusCode::FORBIDDEN, "10007", "admin access required"));
}
```

- [ ] **Step 5: Verify**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo sqlx migrate run -p api
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api auth_roles
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo check -p api
```

Expected: tests and check pass.

- [ ] **Step 6: Commit**

```bash
git add crates/api/migrations/20260520000001_add_user_roles_and_party_roles.sql crates/api/src
git commit -m "feat: add user admin roles"
```

---

### Task 2: Onboarding Domain Schema And Models

**Owner:** Subagent B

**Files:**
- Create: `crates/api/migrations/20260520000002_create_client_onboardings.sql`
- Create: `crates/api/src/models/onboarding.rs`
- Create: `crates/api/src/errors/onboarding_error.rs`
- Modify: `crates/api/src/models/mod.rs`
- Modify: `crates/api/src/errors/mod.rs`

- [ ] **Step 1: Add migration**

Create:

```sql
CREATE TABLE client_onboardings (
    id TEXT PRIMARY KEY,
    submitted_by_user_id TEXT NOT NULL REFERENCES users(id),
    company_name TEXT NOT NULL,
    company_email TEXT,
    phone TEXT,
    country_code TEXT NOT NULL,
    registration_number TEXT,
    address TEXT,
    status TEXT NOT NULL CHECK (
        status IN (
            'draft',
            'kyb_pending',
            'manual_review_pending',
            'approved',
            'rejected',
            'failed'
        )
    ),
    temporal_workflow_id TEXT UNIQUE,
    kyb_vendor_a_status TEXT,
    kyb_vendor_a_response JSONB,
    kyb_vendor_b_status TEXT,
    kyb_vendor_b_response JSONB,
    reviewed_by_user_id TEXT REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    created_party_id TEXT REFERENCES parties(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_client_onboardings_submitted_by ON client_onboardings(submitted_by_user_id);
CREATE INDEX idx_client_onboardings_status ON client_onboardings(status);
CREATE INDEX idx_client_onboardings_workflow_id ON client_onboardings(temporal_workflow_id);
```

- [ ] **Step 2: Add models**

Create Rust structs:

```rust
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ClientOnboarding {
    pub id: String,
    pub submitted_by_user_id: String,
    pub company_name: String,
    pub company_email: Option<String>,
    pub phone: Option<String>,
    pub country_code: String,
    pub registration_number: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub temporal_workflow_id: Option<String>,
    pub kyb_vendor_a_status: Option<String>,
    pub kyb_vendor_a_response: Option<serde_json::Value>,
    pub kyb_vendor_b_status: Option<String>,
    pub kyb_vendor_b_response: Option<serde_json::Value>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub review_note: Option<String>,
    pub created_party_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

Add `CreateClientOnboardingReq` with validator rules:

```rust
company_name: length min 1 max 255
company_email: optional email
phone: optional length min 1 max 50
country_code: length min 2 max 2
registration_number: optional length min 1 max 100
address: optional length min 1 max 500
```

Add `AdminReviewClientOnboardingReq`:

```rust
pub approved: bool,
pub note: Option<String>,
```

- [ ] **Step 3: Add errors**

Create `OnboardingError`:

```rust
NotFound -> 60001
InvalidStatus -> 60002
TemporalStartFailed -> 60003
TemporalSignalFailed -> 60004
KybRejected -> 60005
Database(_) -> 60006
```

- [ ] **Step 4: Verify**

Run: `DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo check -p api`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add crates/api/migrations/20260520000002_create_client_onboardings.sql crates/api/src/models crates/api/src/errors
git commit -m "feat: add client onboarding domain models"
```

---

### Task 3: Backend Onboarding Services

**Owner:** Subagent B

**Files:**
- Create: `crates/api/src/services/onboarding.rs`
- Modify: `crates/api/src/services/mod.rs`
- Modify: `crates/api/src/models/party.rs`
- Modify: `crates/api/src/services/parties.rs`

- [ ] **Step 1: Write service tests**

Create service tests for:

```rust
create_submission_inserts_kyb_pending_with_workflow_id
list_pending_manual_review_returns_only_manual_review_pending
record_admin_decision_rejects_non_pending_status
complete_approved_onboarding_creates_originator_business_party
```

Run: `DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api onboarding_service`

Expected: fail because service does not exist.

- [ ] **Step 2: Implement creation**

Add:

```rust
pub async fn create(
    pool: &PgPool,
    submitted_by_user_id: &str,
    req: CreateClientOnboardingReq,
) -> Result<ClientOnboardingResp, OnboardingError>
```

Behavior:

- Generate `onb_` id with `utils::generate_id("onb")`.
- Set `workflow_id = format!("client-onboarding-{id}")`.
- Insert `status = 'kyb_pending'`.
- Return response.

- [ ] **Step 3: Implement KYB result update**

Add:

```rust
pub async fn record_kyb_results(
    pool: &PgPool,
    onboarding_id: &str,
    vendor_a: serde_json::Value,
    vendor_b: serde_json::Value,
    passed: bool,
) -> Result<ClientOnboardingResp, OnboardingError>
```

Behavior:

- If `passed`, set status `manual_review_pending`.
- If not, set status `rejected`.
- Store each vendor status and response JSON.

- [ ] **Step 4: Implement admin review**

Add:

```rust
pub async fn record_admin_decision(
    pool: &PgPool,
    onboarding_id: &str,
    admin_user_id: &str,
    req: AdminReviewClientOnboardingReq,
) -> Result<ClientOnboardingResp, OnboardingError>
```

Behavior:

- Only allow current status `manual_review_pending`.
- If rejected, set status `rejected`, reviewer fields, note.
- If approved, set reviewer fields and keep status `manual_review_pending` until Temporal completes party creation. This avoids claiming approval before the workflow creates the artifact.

- [ ] **Step 5: Implement final party creation**

Add:

```rust
pub async fn complete_with_originator_party(
    pool: &PgPool,
    onboarding_id: &str,
) -> Result<ClientOnboardingResp, OnboardingError>
```

Behavior inside one SQL transaction:

- Lock onboarding row `FOR UPDATE`.
- Require status `manual_review_pending` and `reviewed_by_user_id IS NOT NULL`.
- Insert `parties` row with `type = 'business'`, `role = 'originator'`.
- Set onboarding status `approved` and `created_party_id`.

- [ ] **Step 6: Verify**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api onboarding_service
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo check -p api
```

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add crates/api/src/services crates/api/src/models/party.rs
git commit -m "feat: add onboarding services"
```

---

### Task 4: Backend Onboarding Controllers And Temporal Boundary

**Owner:** Subagent B, with Subagent C consulted for signal/start contract

**Files:**
- Create: `crates/api/src/controllers/onboarding.rs`
- Modify: `crates/api/src/controllers/mod.rs`
- Modify: `crates/api/src/router.rs`
- Create: `crates/api/src/services/temporal.rs` or `crates/api/src/services/onboarding_worker_client.rs`
- Modify: `crates/api/Cargo.toml`

- [ ] **Step 1: Define internal Temporal boundary**

Choose one implementation path before coding:

- Preferred for speed and maturity: a tiny HTTP bridge in the TypeScript worker app with internal endpoints:
  - `POST /internal/workflows/client-onboarding/start`
  - `POST /internal/workflows/client-onboarding/{id}/review`
- Alternative: direct Rust gRPC Temporal client, if the Rust Temporal client API proves reliable enough for start/signal only.

For the first implementation, use the bridge and add env var:

```text
ONBOARDING_WORKER_URL=http://localhost:4100
```

- [ ] **Step 2: Add user endpoints**

Routes:

```rust
POST /api/v1/onboardings/client
GET /api/v1/onboardings/client
GET /api/v1/onboardings/client/{id}
```

Controller rules:

- Require `AuthUser`.
- Accept `Result<Json<CreateClientOnboardingReq>, JsonRejection>`.
- Validate request before service call.
- After DB insert, call worker start endpoint with onboarding id and workflow id.
- If start fails, mark onboarding `failed` or return `60003` after logging.

- [ ] **Step 3: Add admin endpoints**

Routes:

```rust
GET /api/v1/admin/onboardings
GET /api/v1/admin/onboardings/{id}
POST /api/v1/admin/onboardings/{id}/review
```

Controller rules:

- Require `AdminUser`.
- List defaults to `manual_review_pending`.
- Review body validates `note` length max 1000.
- Call `record_admin_decision`.
- Signal worker review endpoint.
- Return unified API response.

- [ ] **Step 4: Add internal worker callback endpoints**

Routes:

```rust
POST /api/v1/internal/onboardings/{id}/kyb-results
POST /api/v1/internal/onboardings/{id}/complete
```

For local development, protect with `INTERNAL_API_TOKEN` header:

```text
Authorization: Bearer ${INTERNAL_API_TOKEN}
```

These endpoints let the Temporal worker store KYB results and trigger final party creation through Rust services.

- [ ] **Step 5: Verify**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo check -p api
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api onboarding
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add crates/api/src/controllers crates/api/src/router.rs crates/api/src/services crates/api/Cargo.toml
git commit -m "feat: expose client onboarding api"
```

---

### Task 5: Temporal Worker

**Owner:** Subagent C

**Files:**
- Create: `apps/temporal-worker/package.json`
- Create: `apps/temporal-worker/tsconfig.json`
- Create: `apps/temporal-worker/src/workflows/client-onboarding.ts`
- Create: `apps/temporal-worker/src/activities/kyb.ts`
- Create: `apps/temporal-worker/src/activities/api.ts`
- Create: `apps/temporal-worker/src/worker.ts`
- Create: `apps/temporal-worker/src/client.ts`
- Modify: `package.json`

- [ ] **Step 1: Add package dependencies**

Use:

```json
{
  "name": "@rust-fintech/temporal-worker",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "tsx src/worker.ts",
    "bridge": "tsx src/client.ts",
    "check": "tsc --noEmit"
  },
  "dependencies": {
    "@temporalio/activity": "latest",
    "@temporalio/client": "latest",
    "@temporalio/worker": "latest",
    "@temporalio/workflow": "latest",
    "express": "latest",
    "tsx": "latest",
    "zod": "latest"
  },
  "devDependencies": {
    "typescript": "latest"
  }
}
```

- [ ] **Step 2: Implement workflow**

Workflow behavior:

```ts
export async function clientOnboardingWorkflow(input: {
  onboardingId: string;
  workflowId: string;
}) {
  const [vendorA, vendorB] = await Promise.all([
    activities.runVendorA(input),
    activities.runVendorB(input),
  ]);

  const passed = vendorA.companyExists && !vendorA.sanctioned && !vendorA.ofacListed
    && vendorB.companyExists && !vendorB.sanctioned && !vendorB.ofacListed;

  await activities.recordKybResults({ onboardingId: input.onboardingId, vendorA, vendorB, passed });

  if (!passed) {
    return { status: "rejected" };
  }

  const decision = await condition(() => approvalDecision !== undefined);

  if (!approvalDecision?.approved) {
    return { status: "rejected" };
  }

  await activities.completeOnboarding({ onboardingId: input.onboardingId });
  return { status: "approved" };
}
```

Use a signal:

```ts
export const reviewSignal = defineSignal<[AdminReviewSignal]>("adminReview");
```

- [ ] **Step 3: Implement fake KYB activities**

Each vendor waits 10 seconds:

```ts
await new Promise((resolve) => setTimeout(resolve, 10_000));
```

Return deterministic fake pass data based on input fields in activity code. Example:

```ts
{
  vendor: "vendor_a",
  companyExists: true,
  sanctioned: false,
  ofacListed: false,
  referenceId: `fake-a-${onboardingId}`
}
```

- [ ] **Step 4: Implement API callback activities**

Use `fetch` to call Rust internal endpoints:

- `recordKybResults`
- `completeOnboarding`

Include `Authorization: Bearer ${INTERNAL_API_TOKEN}`.

- [ ] **Step 5: Implement bridge**

`src/client.ts` exposes an Express server on `4100`:

- `POST /internal/workflows/client-onboarding/start`
  - Starts workflow with `workflowId`.
  - Uses task queue `client-onboarding`.
- `POST /internal/workflows/client-onboarding/:onboardingId/review`
  - Gets workflow handle by `client-onboarding-{onboardingId}`.
  - Sends `adminReview` signal.

- [ ] **Step 6: Verify**

Run in separate terminals:

```bash
temporal server start-dev
npm run worker:dev
npm run api:dev
```

Then submit onboarding and approve through curl.

Expected:

- Onboarding becomes `kyb_pending`.
- About 10 seconds later it becomes `manual_review_pending`.
- Admin approval signals workflow.
- Onboarding becomes `approved`.
- A `parties` row exists with `type = 'business'` and `role = 'originator'`.

- [ ] **Step 7: Commit**

```bash
git add apps/temporal-worker package.json package-lock.json
git commit -m "feat: add temporal onboarding worker"
```

---

### Task 6: OpenAPI And Generated Client

**Owner:** Subagent D

**Files:**
- Create or modify: `docs/openapi.yaml`
- Create: `orval.config.ts`
- Create: `packages/api-client/package.json`
- Create: `packages/api-client/src/index.ts`
- Modify: `package.json`

- [ ] **Step 1: Define OpenAPI paths**

Document:

```yaml
/api/v1/onboardings/client:
  post:
    operationId: createClientOnboarding
  get:
    operationId: listMyClientOnboardings
/api/v1/onboardings/client/{id}:
  get:
    operationId: getMyClientOnboarding
/api/v1/admin/onboardings:
  get:
    operationId: listAdminOnboardings
/api/v1/admin/onboardings/{id}:
  get:
    operationId: getAdminOnboarding
/api/v1/admin/onboardings/{id}/review:
  post:
    operationId: reviewClientOnboarding
```

Use the unified `ApiResponse` envelope for all responses.

- [ ] **Step 2: Configure Orval**

Generate:

- typed request/response interfaces
- endpoint functions
- TanStack Query hooks/options

Target:

```text
packages/api-client/src/generated
```

- [ ] **Step 3: Replace hand-written frontend API types where practical**

Leave `apps/web/src/lib/auth-api.ts` only if auth endpoints are not yet fully represented in OpenAPI. Otherwise migrate auth to generated client too.

- [ ] **Step 4: Verify**

Run:

```bash
npm run api-client:generate
npm run web:check
```

Expected: generated client compiles and frontend imports resolve.

- [ ] **Step 5: Commit**

```bash
git add docs/openapi.yaml orval.config.ts packages/api-client package.json package-lock.json apps/web/src
git commit -m "feat: generate typed onboarding api client"
```

---

### Task 7: User Client Onboarding Wizard

**Owner:** Subagent E

**Files:**
- Create: `apps/web/src/routes/client-onboarding.tsx`
- Modify: `apps/web/src/router.tsx`
- Modify: `apps/web/src/routes/dashboard.tsx`
- Modify: `apps/web/src/styles.css`

- [ ] **Step 1: Add route**

Add route:

```ts
path: "/onboarding/client"
component: ClientOnboardingRoute
```

- [ ] **Step 2: Build wizard with TanStack Form**

Steps:

1. Company identity: company name, country code, registration number.
2. Contact details: company email, phone, address.
3. Review and submit.

Validation mirrors backend:

```ts
companyName.trim().length > 0
countryCode.length === 2
companyEmail optional but must include "@"
```

- [ ] **Step 3: Submit via generated mutation**

On success, show status panel:

```text
KYB in progress
Manual review pending
Approved
Rejected
```

Poll the onboarding detail query every 5 seconds while status is `kyb_pending` or `manual_review_pending`.

- [ ] **Step 4: Add dashboard entry point**

Add a primary action in the dashboard sidebar or main dashboard:

```text
Onboard Client
```

No trailing slashes in API URLs.

- [ ] **Step 5: Verify**

Run:

```bash
npm run web:check
npm run web:dev
```

Use browser verification at `http://localhost:5173/onboarding/client`.

Expected: form fits desktop/mobile, submit creates onboarding, polling displays status.

- [ ] **Step 6: Commit**

```bash
git add apps/web/src
git commit -m "feat: add client onboarding wizard"
```

---

### Task 8: Admin Portal

**Owner:** Subagent F

**Files:**
- Create: `apps/web/src/routes/admin-onboarding.tsx`
- Modify: `apps/web/src/router.tsx`
- Modify: `apps/web/src/routes/dashboard.tsx`
- Modify: `apps/web/src/styles.css`

- [ ] **Step 1: Add route**

Add route:

```ts
path: "/admin/onboarding"
component: AdminOnboardingRoute
```

- [ ] **Step 2: Build admin queue**

Use TanStack Query to list:

```ts
status=manual_review_pending
```

Display:

- Company name
- Country
- Submitted by
- Vendor A result
- Vendor B result
- Created time
- Current status

- [ ] **Step 3: Build review controls**

For each selected onboarding:

- Approve button
- Reject button
- Optional note input

Use mutation:

```ts
reviewClientOnboarding(id, { approved: true, note })
reviewClientOnboarding(id, { approved: false, note })
```

Invalidate admin queue and detail queries after success.

- [ ] **Step 4: Hide admin nav for non-admins**

Use `/auth/me` role from `UserResp`. Since `leejayhsu@gmail.com` is seeded admin, that account should see admin navigation.

- [ ] **Step 5: Verify**

Run:

```bash
npm run web:check
npm run web:dev
```

Use browser verification at `http://localhost:5173/admin/onboarding`.

Expected:

- Non-admin receives forbidden API response and no admin nav.
- Admin sees pending onboarding.
- Approve creates party after workflow signal.

- [ ] **Step 6: Commit**

```bash
git add apps/web/src
git commit -m "feat: add onboarding admin portal"
```

---

### Task 9: End-To-End Integration Verification

**Owner:** Subagent G

**Files:**
- Create: `docs/onboarding-temporal-dev.md`
- Optionally create: `scripts/smoke-onboarding.sh`
- Modify: any files needed for cross-agent compile fixes only.

- [ ] **Step 1: Document local startup**

Create docs with:

```bash
temporal server start-dev
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo run -p api
npm run worker:dev
npm run web:dev
```

Include required env vars:

```text
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech
ONBOARDING_WORKER_URL=http://localhost:4100
API_BASE_URL=http://localhost:3000
TEMPORAL_ADDRESS=localhost:7233
TEMPORAL_NAMESPACE=default
INTERNAL_API_TOKEN=dev-internal-token
COOKIE_SECURE=false
```

- [ ] **Step 2: Smoke test full flow**

Use the seeded/admin account:

```text
leejayhsu@gmail.com
```

Run:

1. Sign in.
2. Submit onboarding wizard.
3. Confirm DB status `kyb_pending`.
4. Wait 10-15 seconds.
5. Confirm DB status `manual_review_pending`.
6. Open admin portal.
7. Approve.
8. Confirm DB status `approved`.
9. Confirm `created_party_id` is set.
10. Confirm `parties.role = 'originator'`.

- [ ] **Step 3: Run full verification**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo test -p api
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech cargo check -p api
npm run api-client:generate
npm run web:check
npm run worker:check
```

Expected: all pass.

- [ ] **Step 4: Browser verification**

Use desktop and mobile viewports for:

- `/dashboard`
- `/onboarding/client`
- `/admin/onboarding`

Expected:

- No overlapping text.
- Wizard controls remain usable.
- Admin review controls fit on mobile.
- Status changes are visible after polling.

- [ ] **Step 5: Commit**

```bash
git add docs scripts apps crates packages package.json package-lock.json
git commit -m "test: verify temporal onboarding flow"
```

---

## Execution Order

1. Subagent A implements Task 1.
2. Subagent B implements Tasks 2-4.
3. Subagent C implements Task 5 in parallel after Task 4 contract is agreed.
4. Subagent D implements Task 6 after Tasks 2-4 define final API shapes.
5. Subagent E implements Task 7 after Task 6 generates client hooks.
6. Subagent F implements Task 8 after Task 6 generates client hooks.
7. Subagent G performs Task 9 after all feature workers complete.

## Risk Register

- Temporal Rust SDK instability: use TypeScript SDK for workflow code and keep Rust integration at a narrow HTTP bridge boundary.
- Race between admin approval and workflow state: admin endpoint only allows approval when DB status is `manual_review_pending`; workflow waits for signal after recording KYB results.
- Party taxonomy confusion: preserve `business` as entity type and introduce `originator` as role.
- Fake vendor timing in tests: unit-test activity logic without waiting 10 seconds; use the 10-second delay only in integration/dev activities.
- Security of internal callbacks: use `INTERNAL_API_TOKEN` for local implementation; later replace with mTLS or private network auth.

## Done Criteria

- A normal user can submit a business client onboarding wizard.
- Temporal starts a workflow for that onboarding.
- Two fake KYB vendors run concurrently and take about 10 seconds total, not 20 seconds.
- Passing KYB moves onboarding to manual review.
- Admin user `leejayhsu@gmail.com` can approve the onboarding.
- Approval causes Temporal to complete the workflow and create a `party` with `type = 'business'` and `role = 'originator'`.
- Non-admin users cannot access admin approval APIs.
- OpenAPI spec and generated TypeScript client match the Rust API.
- `cargo test -p api`, `cargo check -p api`, `npm run web:check`, and `npm run worker:check` pass.

## Source Notes

- Temporal docs SDK chooser lists TypeScript as a primary SDK option, while not listing Rust in that chooser.
- `temporalio-sdk` crate docs currently describe the Rust SDK as alpha-stage and the workflow API as unstable, which is why this plan uses TypeScript for workflow execution.
