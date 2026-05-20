# Login Landing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a polished landing page with a working login form backed by the existing Rust auth API.

**Architecture:** The frontend root route becomes the unauthenticated landing/login screen. Auth requests live in a small API module that understands the existing unified API response format and uses cookies for sessions. A separate dashboard route remains available after successful signin.

**Tech Stack:** React, TypeScript, TanStack Router, TanStack Form, TanStack Query, shadcn-style local UI components, existing Axum `/api/v1/auth/*` routes.

---

### Task 1: Add Auth Client

**Files:**
- Create: `apps/web/src/lib/auth-api.ts`

- [x] Define `signin`, `signup`, and `me` helpers using `fetch` with `credentials: "include"` and the backend response envelope.
- [x] Surface backend `error.desc` messages as thrown `Error` instances for TanStack Query mutations.

### Task 2: Add shadcn UI Primitives

**Files:**
- Create: `apps/web/src/components/ui/input.tsx`
- Create: `apps/web/src/components/ui/label.tsx`
- Create: `apps/web/src/components/ui/card.tsx`
- Create: `apps/web/src/components/ui/alert.tsx`

- [x] Add local shadcn-compatible primitives matching the existing `Button` conventions.

### Task 3: Build Landing/Login Route

**Files:**
- Create: `apps/web/src/routes/landing.tsx`
- Modify: `apps/web/src/router.tsx`

- [x] Use the approved split landing/login visual direction.
- [x] Use TanStack Form for email/password validation.
- [x] Try signin first; if the test user does not exist, offer a one-click signup fallback with the same credentials.
- [x] Navigate to `/dashboard` after successful signin or signup.

### Task 4: Keep Dashboard Separate

**Files:**
- Modify: `apps/web/src/router.tsx`
- Modify: `apps/web/src/routes/dashboard.tsx`

- [x] Move the current dashboard from `/` to `/dashboard`.
- [x] Add a simple signed-in header affordance without building a full auth shell yet.

### Task 5: Style And Verify

**Files:**
- Modify: `apps/web/src/styles.css`

- [x] Replace broad element-level button/input styling with layout-focused styles so shadcn components control form appearance.
- [x] Run frontend checks/build.
- [x] Start the web app and inspect the page in browser if available.
