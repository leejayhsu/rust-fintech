# Temporal Client Onboarding Development

## Required Environment

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_fintech
export ONBOARDING_WORKER_URL=http://localhost:4100
export API_BASE_URL=http://localhost:3000
export TEMPORAL_ADDRESS=localhost:7233
export TEMPORAL_NAMESPACE=default
export INTERNAL_API_TOKEN=dev-internal-token
export COOKIE_SECURE=false
```

`INTERNAL_API_TOKEN` is required. The API fails closed when it is missing or blank.

## Local Startup

Run each command in its own terminal:

```bash
npm run temporal:dev
DATABASE_URL=$DATABASE_URL COOKIE_SECURE=false INTERNAL_API_TOKEN=$INTERNAL_API_TOKEN npm run api:dev
INTERNAL_API_TOKEN=$INTERNAL_API_TOKEN API_BASE_URL=$API_BASE_URL npm run worker:dev
npm run web:dev
```

The worker process also starts the local HTTP bridge on port `4100`.

## Smoke Flow

1. Sign in as `leejayhsu@gmail.com`.
2. Open `/onboarding/client`.
3. Submit a business client.
4. Confirm the onboarding starts as `kyb_pending`.
5. Wait about 10-15 seconds for both fake KYB activities to complete concurrently.
6. Confirm status moves to `manual_review_pending`.
7. Open `/admin/onboarding`.
8. Approve the onboarding.
9. Confirm the workflow completes and status becomes `approved`.
10. Confirm `created_party_id` is set and the created party has `type = 'business'` and `role = 'originator'`.

## Verification Commands

```bash
DATABASE_URL=$DATABASE_URL cargo check -p api
npm run api-client:generate
npm run api-client:check
npm run worker:check
npm run web:check
npm --workspace @rust-fintech/web run typecheck
```
