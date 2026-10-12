# Vertical Slices & Platform Subsystems Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor CMS into Domain Vertical Slices by extracting `crates/cms-notifications` (with models, DB queries, service, WebSocket live hub, and Axum HTTP routes) and implementing the six core platform subsystems from the MedBrains architecture: Transactional Outbox (`cms-outbox`), Dev Seed Fixtures (`cms-seed`), Onboarding Verification (`email_verification_tokens`), Workspace Invitations (`workspace_invitations`), Cron Scheduling (`cms-scheduler`), and Report Runs (`report_runs`).

**Architecture:** Each vertical slice crate (`cms-notifications`) and platform infrastructure crate (`cms-outbox`, `cms-seed`, `cms-scheduler`) is fully self-contained, compile-time verified via SQLx, and adheres to strict Rust 2024 edition guidelines (`forbid(unsafe_code)`, no unwraps or panics, structured error propagation with `AppError` and `?`). Outbox acts as the sole delivery backbone for transactional emails and external side effects.

**Tech Stack:** Rust 2024 (MSRV 1.85), Axum 0.8, Tokio 1.x, SQLx 0.9 (PostgreSQL 14+ native `gen_random_uuid()`), Serde / Serde JSON, Chrono, Uuid, Argon2.

## Global Constraints

- Strict Rust edition 2024 (MSRV 1.85).
- `unsafe` code is strictly forbidden (`forbid(unsafe_code)`).
- Zero placeholders: no `TODO`, no stubs, no fake mocks, no incomplete skeletons. Every implementation must be 100% complete, fully wired, tested, and compile cleanly.
- Strict Clippy rules: `clippy::all` denied. Denied lints: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr`, `wildcard_imports`.
- Error propagation must use `Result<T, AppError>` and `?`.
- Strict Internal Platform scope: No billing, no payment gateways, no pricing tiers, no marketing landing pages.
- Cargo builds on Windows must use `-j 2` to avoid stack buffer overrun during heavy proc-macro expansion.

---

### Task 1: `crates/cms-notifications` Vertical Slice Crate

**Files:**
- Create: `crates/cms-notifications/Cargo.toml`
- Create: `crates/cms-notifications/src/lib.rs`
- Create: `crates/cms-notifications/src/hub.rs`
- Create: `crates/cms-notifications/src/queries.rs`
- Create: `crates/cms-notifications/src/service.rs`
- Create: `crates/cms-notifications/src/handlers.rs`
- Create: `crates/cms-notifications/tests/notifications_test.rs`
- Modify: `Cargo.toml:18-42` (add workspace dependency)
- Modify: `crates/cms-api/src/lib.rs:24-70` (route delegation)

**Interfaces:**
- Consumes: `cms_db`, `cms_entity::notification`, `cms_error::AppError`, `cms_middleware::app_state::AppState`
- Produces: `cms_notifications::router(state: Arc<AppState>) -> axum::Router`, `cms_notifications::NotificationHub`, `cms_notifications::NotificationService`

- [ ] **Step 1: Create `crates/cms-notifications/Cargo.toml`**
Configure package dependencies including `axum`, `tokio`, `futures`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `tracing`, `cms-db`, `cms-entity`, `cms-error`, `cms-middleware`.

- [ ] **Step 2: Implement `crates/cms-notifications/src/hub.rs`**
Implement `NotificationHub` providing a thread-safe broadcast manager (`tokio::sync::broadcast`) for real-time WebSocket client subscriptions keyed by `user_id`. Provide `subscribe(user_id)` and `publish(user_id, event)` methods.

- [ ] **Step 3: Implement `crates/cms-notifications/src/queries.rs`**
Implement PostgreSQL query methods for fetching notifications by user, counting unread notifications, creating new notifications, marking as read, and deleting notifications using `sqlx::query_as!` / `sqlx::QueryBuilder`.

- [ ] **Step 4: Implement `crates/cms-notifications/src/service.rs`**
Implement `NotificationService` coordinating database transactions, status mutations, unread count caching, and publishing real-time events to `NotificationHub`.

- [ ] **Step 5: Implement `crates/cms-notifications/src/handlers.rs` and `lib.rs`**
Implement Axum handlers for `GET /`, `GET /{id}`, `POST /read`, `POST /read-all`, `POST /{id}/archive`, `GET /count`, `GET /unread-count`, and `GET /ws` (WebSocket connection upgrading to live event stream). Export `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 6: Write unit & integration tests in `crates/cms-notifications/tests/notifications_test.rs`**
Test hub subscriptions, message fan-out, query builder SQL generation, and service methods.

- [ ] **Step 7: Wire into root `Cargo.toml` and verify compilation**
Run: `cargo test -p cms-notifications -j 2`
Expected: PASS with 0 errors.

---

### Task 2: Subsystem 1: Transactional Outbox (`cms-outbox`)

**Files:**
- Create: `migrations/0020_outbox.sql`
- Create: `crates/cms-outbox/Cargo.toml`
- Create: `crates/cms-outbox/src/lib.rs`
- Create: `crates/cms-outbox/src/queue.rs`
- Create: `crates/cms-outbox/src/backoff.rs`
- Create: `crates/cms-outbox/src/handler.rs`
- Create: `crates/cms-outbox/src/worker.rs`
- Create: `crates/cms-outbox/src/metrics.rs`
- Create: `crates/cms-outbox/src/handlers/mod.rs`
- Create: `crates/cms-outbox/src/handlers/email.rs`
- Create: `crates/cms-outbox/src/handlers/webhook.rs`
- Create: `crates/cms-outbox/tests/outbox_test.rs`
- Modify: `Cargo.toml` (add `cms-outbox` workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_mailer`, `cms_error::AppError`
- Produces: `cms_outbox::queue_in_tx(&mut Transaction, OutboxRow) -> Result<(), OutboxError>`, `cms_outbox::OutboxWorker`, `cms_outbox::Handler`, `cms_outbox::Registry`

- [ ] **Step 1: Write `migrations/0020_outbox.sql`**
Define `outbox_events` and `outbox_dlq` tables with eligible indices, attempt counters, and timestamp tracking.

- [ ] **Step 2: Create `crates/cms-outbox/Cargo.toml`**
Declare `cms-outbox` with dependencies on `cms-db`, `cms-mailer`, `cms-error`, `tokio`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `tracing`, `async-trait`.

- [ ] **Step 3: Implement `queue.rs` and `backoff.rs`**
Implement `OutboxRow` and `queue_in_tx(tx, row)`. Implement exponential backoff calculation with jitter: `min(2^n * 1s + jitter, 1h)`.

- [ ] **Step 4: Implement `handler.rs`, `metrics.rs`, and built-in handlers**
Define `Handler` trait, `HandlerCtx`, `HandlerError`, and thread-safe `Registry`. Implement `EmailHandler` (delegates to `cms-mailer`) and `WebhookHandler` (HTTP POST).

- [ ] **Step 5: Implement `worker.rs` and `lib.rs`**
Implement `OutboxWorker` loop: polling with `FOR UPDATE SKIP LOCKED`, dispatching to registry, deletion on success, retry backoff on failure, migration to `outbox_dlq` on 10 failures, and stale claim reaper.

- [ ] **Step 6: Write tests in `crates/cms-outbox/tests/outbox_test.rs`**
Test backoff calculation, registry dispatch, error categorization, and queue insertion.

- [ ] **Step 7: Verify with Cargo**
Run: `cargo test -p cms-outbox -j 2`
Expected: PASS with 0 errors.

---

### Task 3: Subsystem 2: Seed & Dev Fixtures (`cms-seed`)

**Files:**
- Create: `crates/cms-seed/Cargo.toml`
- Create: `crates/cms-seed/src/lib.rs`
- Create: `crates/cms-seed/src/users.rs`
- Create: `crates/cms-seed/src/workspace.rs`
- Create: `crates/cms-seed/src/projects.rs`
- Create: `crates/cms-seed/src/pages.rs`
- Create: `crates/cms-seed/src/members.rs`
- Create: `crates/cms-seed/src/bin/cms_seed.rs`
- Create: `crates/cms-seed/tests/seed_test.rs`
- Modify: `Cargo.toml` (add `cms-seed` workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_entity`, `cms_error::AppError`, `argon2`
- Produces: `cms_seed::seed_dev(pool: &PgPool) -> Result<SeedResult, AppError>`, `cms_seed::is_seeded(pool: &PgPool) -> Result<bool, AppError>`

- [ ] **Step 1: Create `crates/cms-seed/Cargo.toml`**
Declare `cms-seed` with dependencies on `cms-db`, `cms-entity`, `cms-error`, `argon2`, `sqlx`, `tokio`, `uuid`, `chrono`, `tracing`, `clap`.

- [ ] **Step 2: Implement `users.rs` and `workspace.rs`**
Implement deterministic user generation (admin, alice, bob) using `Uuid::new_v5` and Argon2 password hashing. Implement default workspace creation.

- [ ] **Step 3: Implement `projects.rs`, `pages.rs`, and `members.rs`**
Implement demo projects with markdown documentation pages and workspace memberships.

- [ ] **Step 4: Implement `lib.rs` and `bin/cms_seed.rs`**
Implement `seed_dev` with idempotency guard (`is_seeded`), summary printer, and CLI binary with `--profile` options.

- [ ] **Step 5: Write tests in `crates/cms-seed/tests/seed_test.rs`**
Verify deterministic UUID generation, password verification against Argon2 hashes, and idempotency logic.

- [ ] **Step 6: Verify with Cargo**
Run: `cargo test -p cms-seed -j 2`
Expected: PASS with 0 errors.

---

### Task 4: Subsystem 3: Onboarding Verification (`email_verification_tokens`)

**Files:**
- Create: `migrations/0021_email_verification_tokens.sql`
- Create: `crates/cms-db/src/onboarding.rs`
- Create: `crates/cms-biz/src/onboarding.rs`
- Modify: `crates/cms-db/src/lib.rs`
- Modify: `crates/cms-biz/src/lib.rs`
- Modify: `crates/cms-api/src/setup/handlers.rs`
- Create: `crates/cms-biz/tests/onboarding_test.rs`

**Interfaces:**
- Consumes: `cms_db`, `cms_outbox`, `sha2::Sha256`
- Produces: `generate_verification_token`, `consume_verification_token`, `setup_status`, `complete_setup_with_token`

- [ ] **Step 1: Write `migrations/0021_email_verification_tokens.sql`**
Create `email_verification_tokens` table with `token_hash`, `purpose`, `expires_at`, and `used_at`.

- [ ] **Step 2: Implement `crates/cms-db/src/onboarding.rs`**
Implement DB operations: insert token hash, find valid unused token by hash, mark token used.

- [ ] **Step 3: Implement `crates/cms-biz/src/onboarding.rs`**
Implement business service: crypto-secure token generation (raw 32-byte hex), SHA-256 hash storage, outbox event queuing inside TX for verification link.

- [ ] **Step 4: Update `cms-api/src/setup/handlers.rs`**
Implement endpoints: `GET /api/setup/status`, `POST /api/setup/complete`, `GET /api/auth/verify-email`, `POST /api/auth/resend-verification`.

- [ ] **Step 5: Write integration tests**
Verify token issuance, verification link validation, expiration enforcement, and single-use guarantee.

---

### Task 5: Subsystem 4: Workspace Invitations (`workspace_invitations`)

**Files:**
- Create: `migrations/0022_workspace_invitations.sql`
- Create: `crates/cms-db/src/invitation.rs`
- Create: `crates/cms-biz/src/invitation.rs`
- Create: `crates/cms-api/src/invitation/mod.rs`
- Create: `crates/cms-api/src/invitation/handlers.rs`
- Modify: `crates/cms-db/src/lib.rs`
- Modify: `crates/cms-biz/src/lib.rs`
- Modify: `crates/cms-api/src/lib.rs`
- Create: `crates/cms-biz/tests/invitation_test.rs`

**Interfaces:**
- Consumes: `cms_db`, `cms_outbox`, `cms_auth`
- Produces: `create_invitation`, `accept_invitation`, `revoke_invitation`, `list_pending_invitations`

- [ ] **Step 1: Write `migrations/0022_workspace_invitations.sql`**
Create `workspace_invitations` table with unique pending email index and token hash index.

- [ ] **Step 2: Implement `crates/cms-db/src/invitation.rs`**
Implement DB queries: insert invitation, find by token hash, list pending by workspace, mark accepted, mark revoked.

- [ ] **Step 3: Implement `crates/cms-biz/src/invitation.rs`**
Implement invitation lifecycle: invite creation, transactional outbox email enqueue, token acceptance (with user creation / workspace linkage).

- [ ] **Step 4: Implement Axum endpoints in `crates/cms-api/src/invitation/`**
Routes: `GET /api/app/invitations`, `POST /api/app/invitations`, `DELETE /api/app/invitations/:id`, `GET /api/app/invitations/accept`, `POST /api/app/invitations/accept`.

- [ ] **Step 5: Write integration tests**
Verify invitation token generation, outbox event creation, acceptance, and role assignment.

---

### Task 6: Subsystem 5: Scheduling (`cms-scheduler` & `CmsJob.scheduled_at`)

**Files:**
- Create: `migrations/0023_scheduled_jobs.sql`
- Create: `crates/cms-scheduler/Cargo.toml`
- Create: `crates/cms-scheduler/src/lib.rs`
- Create: `crates/cms-scheduler/src/cron.rs`
- Create: `crates/cms-scheduler/src/registry.rs`
- Create: `crates/cms-scheduler/src/runner.rs`
- Modify: `crates/cms-queue/src/postgres.rs` (add `scheduled_at` support and `enqueue_at`)
- Create: `crates/cms-scheduler/tests/scheduler_test.rs`
- Modify: `Cargo.toml` (add `cms-scheduler` workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_queue`, `cron`
- Produces: `Scheduler::spawn`, `CronJobSpec`, `enqueue_at`

- [ ] **Step 1: Write `migrations/0023_scheduled_jobs.sql`**
Add `scheduled_at` column to `CmsJob` with partial index. Create `cms_scheduled_jobs` table.

- [ ] **Step 2: Update `crates/cms-queue/src/postgres.rs`**
Update job dequeue query to enforce `(scheduled_at IS NULL OR scheduled_at <= now())`. Add `enqueue_at` method.

- [ ] **Step 3: Create `crates/cms-scheduler/Cargo.toml`**
Declare `cms-scheduler` with dependencies on `cms-db`, `cms-queue`, `cms-error`, `cron`, `tokio`, `chrono`, `sqlx`, `tracing`.

- [ ] **Step 4: Implement `cron.rs`, `registry.rs`, and `runner.rs`**
Implement cron parser, system recurring jobs registry (session purge, token purge, report purge), and scheduler tick loop with `FOR UPDATE SKIP LOCKED`.

- [ ] **Step 5: Write tests and verify compilation**
Run: `cargo test -p cms-scheduler -j 2`
Expected: PASS with 0 errors.

---

### Task 7: Subsystem 6: Report Runs (`report_runs`)

**Files:**
- Create: `migrations/0024_report_runs.sql`
- Create: `crates/cms-db/src/report.rs`
- Create: `crates/cms-biz/src/report.rs`
- Modify: `crates/cms-api/src/export/handlers.rs` (extend with report runs routes)
- Modify: `crates/cms-db/src/lib.rs`
- Modify: `crates/cms-biz/src/lib.rs`
- Create: `crates/cms-biz/tests/report_test.rs`

**Interfaces:**
- Consumes: `cms_db`, `cms_queue`, `cms_storage`
- Produces: `create_report_run`, `execute_report_run`, `list_report_runs`, `get_report_download_url`

- [ ] **Step 1: Write `migrations/0024_report_runs.sql`**
Create `report_runs` table with workspace index, expiration index, status transitions.

- [ ] **Step 2: Implement `crates/cms-db/src/report.rs`**
Implement DB methods: insert report run, update status (running, done, failed), get by ID, list by workspace, cleanup expired.

- [ ] **Step 3: Implement `crates/cms-biz/src/report.rs`**
Implement report run orchestration: enqueue export background job, process export payload, upload to storage, generate download URL.

- [ ] **Step 4: Implement Axum endpoints in `cms-api/src/export/`**
Routes: `GET /api/app/reports`, `POST /api/app/reports`, `GET /api/app/reports/:id`, `GET /api/app/reports/:id/download`, `DELETE /api/app/reports/:id`.

- [ ] **Step 5: Write integration tests**
Verify lifecycle from creation to completion, presigned download URL generation, and cleanup.

---

### Task 8: End-to-End Verification & Documentation

**Files:**
- Modify: `AGENTS.md` (document all new subsystems and crates)
- Modify: `Cargo.toml` (finalize all dependencies and members)

- [ ] **Step 1: Run comprehensive Clippy and test suite**
Run: `cargo clippy --all-targets --all-features -j 2`
Expected: Clean compilation with 0 warnings/errors.
Run: `cargo test --all-targets -j 2`
Expected: All tests pass.

- [ ] **Step 2: Update `AGENTS.md` and commit**
Document crate structure, public API contracts, and usage guidelines.
