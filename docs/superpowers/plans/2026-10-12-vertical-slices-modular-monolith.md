# Vertical Slices & Shared Services Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the next phase of the Modular Monolith architecture adapted from MedBrains: extract `crates/cms-server-services` (shared cross-domain services), followed by self-contained vertical slice crates for Invitations (`crates/cms-invitations`), Onboarding (`crates/cms-onboarding`), and Reports (`crates/cms-reports`).

**Architecture:** Following MedBrains `RFC-SERVER-CRATE-SPLIT.md`:
1. `crates/cms-server-services` serves as the shared services layer preventing circular dependencies among leaves.
2. Each domain crate (`cms-invitations`, `cms-onboarding`, `cms-reports`) is a true self-contained vertical slice containing its database queries, business services, Axum HTTP router (`pub fn router(state: Arc<AppState>) -> Router`), and integration tests.
3. `cms-api` delegates routes to each domain vertical slice router with zero behavior regression.

**Tech Stack:** Rust 2024 (MSRV 1.85), Axum 0.8, Tower 0.5, Tokio 1.0, SQLx 0.9, Argon2, Sha2, Serde/Serde JSON, Uuid, Chrono.

## Global Constraints

- Strict Rust Edition 2024 (MSRV 1.85) with `forbid(unsafe_code)`.
- Clippy compliance: `clippy::all` denied. No `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`.
- Absolute zero placeholders: No mocks, stubs, or fake comments. Every query, function, handler, and test is 100% complete and working.
- Internal platform scope: Strictly zero billing, payment gateways, or pricing (ADR 0001).
- Windows parallel build constraint: Run compilation/tests with `-j 2`.
- Git commits on Windows: Must use `--no-verify` to bypass hanging pre-commit bash scripts.

---

### Task 1: Create `crates/cms-server-services` Crate

**Files:**
- Create: `crates/cms-server-services/Cargo.toml`
- Create: `crates/cms-server-services/src/lib.rs`
- Create: `crates/cms-server-services/src/auth_audit.rs`
- Create: `crates/cms-server-services/src/storage_url.rs`
- Create: `crates/cms-server-services/src/tenant_context.rs`
- Create: `crates/cms-server-services/src/event_dispatcher.rs`
- Create: `crates/cms-server-services/tests/services_test.rs`
- Modify: `Cargo.toml` (add workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_entity`, `cms_error::AppError`, `cms_server_core::app_state::AppState`, `cms_outbox`, `cms_storage`
- Produces: `cms_server_services::auth_audit::record_security_audit`, `cms_server_services::storage_url::resolve_asset_url`, `cms_server_services::tenant_context::resolve_workspace_scope`, `cms_server_services::event_dispatcher::dispatch_domain_event`

- [ ] **Step 1: Create `crates/cms-server-services/Cargo.toml`**
  Declare package and dependencies (`tokio`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `tracing`, `sha2`, `hex`, `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-outbox`, `cms-storage`).

- [ ] **Step 2: Register in root `Cargo.toml`**
  Add `cms-server-services = { path = "crates/cms-server-services" }` to `[workspace.dependencies]` and `"crates/cms-server-services"` to `[workspace.members]`.

- [ ] **Step 3: Implement `auth_audit.rs`**
  Implement cross-domain audit event helpers utilizing `cms_db::AuditLogger` and SHA-256 hash chaining.

- [ ] **Step 4: Implement `storage_url.rs`**
  Implement storage asset URL resolvers and presigned download URL builders for export artifacts and media assets.

- [ ] **Step 5: Implement `tenant_context.rs` & `event_dispatcher.rs`**
  Implement workspace membership verification helpers and transactional outbox event dispatch wrappers.

- [ ] **Step 6: Write unit & integration tests in `crates/cms-server-services/tests/services_test.rs`**
  Verify audit logging recording, storage key URL formatting, and event dispatch logic.

- [ ] **Step 7: Verify compilation & commit**
  Run: `cargo test -p cms-server-services -j 2`
  Git add & commit: `feat(server-services): implement cms-server-services shared services crate`

---

### Task 2: Extract `crates/cms-invitations` Vertical Slice Crate

**Files:**
- Create: `crates/cms-invitations/Cargo.toml`
- Create: `crates/cms-invitations/src/lib.rs`
- Create: `crates/cms-invitations/src/models.rs`
- Create: `crates/cms-invitations/src/queries.rs`
- Create: `crates/cms-invitations/src/service.rs`
- Create: `crates/cms-invitations/src/handlers.rs`
- Create: `crates/cms-invitations/tests/invitations_test.rs`
- Modify: `crates/cms-api/src/invitation/mod.rs` (delegate to `cms_invitations::router`)
- Modify: `Cargo.toml` (add workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_error::AppError`, `cms_server_core::app_state::AppState`, `cms_server_services`, `cms_outbox`
- Produces: `cms_invitations::router(state: Arc<AppState>) -> axum::Router`, `cms_invitations::InvitationService`, `cms_invitations::InvitationQueries`

- [ ] **Step 1: Create `crates/cms-invitations/Cargo.toml`**
  Declare dependencies on `axum`, `tokio`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `sha2`, `hex`, `argon2`, `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-server-services`, `cms-outbox`.

- [ ] **Step 2: Register in root `Cargo.toml`**
  Add `cms-invitations = { path = "crates/cms-invitations" }` to `[workspace.dependencies]` and `"crates/cms-invitations"` to `[workspace.members]`.

- [ ] **Step 3: Implement `models.rs` and `queries.rs`**
  Define request/response DTOs and compile-time checked database operations: insert invitation, find by token hash, list pending, mark accepted/revoked.

- [ ] **Step 4: Implement `service.rs`**
  Implement `InvitationService`: cryptographic token generation (SHA-256 hash), outbox transactional dispatch for `email.invitation`, and acceptance flow (user creation, workspace member link, token status mutation).

- [ ] **Step 5: Implement `handlers.rs` & `lib.rs`**
  Implement Axum handlers for `GET /api/app/invitations`, `POST /api/app/invitations`, `DELETE /api/app/invitations/{id}`, `GET /api/app/invitations/accept`, and `POST /api/app/invitations/accept`. Export `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 6: Write tests in `crates/cms-invitations/tests/invitations_test.rs`**
  Test token hashing, query validation, acceptance rules, and router construction.

- [ ] **Step 7: Delegate `crates/cms-api/src/invitation/mod.rs` to `cms-invitations`**
  Update `cms-api` router delegation to mount `cms_invitations::router(state)`.

- [ ] **Step 8: Verify compilation & commit**
  Run: `cargo test -p cms-invitations -j 2`
  Git add & commit: `feat(invitations): extract cms-invitations vertical slice crate`

---

### Task 3: Extract `crates/cms-onboarding` Vertical Slice Crate

**Files:**
- Create: `crates/cms-onboarding/Cargo.toml`
- Create: `crates/cms-onboarding/src/lib.rs`
- Create: `crates/cms-onboarding/src/models.rs`
- Create: `crates/cms-onboarding/src/queries.rs`
- Create: `crates/cms-onboarding/src/service.rs`
- Create: `crates/cms-onboarding/src/handlers.rs`
- Create: `crates/cms-onboarding/tests/onboarding_test.rs`
- Modify: `crates/cms-api/src/setup/mod.rs` (delegate to `cms_onboarding::router`)
- Modify: `Cargo.toml` (add workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_error::AppError`, `cms_server_core::app_state::AppState`, `cms_server_services`, `cms_outbox`
- Produces: `cms_onboarding::router(state: Arc<AppState>) -> axum::Router`, `cms_onboarding::OnboardingService`

- [ ] **Step 1: Create `crates/cms-onboarding/Cargo.toml`**
  Declare dependencies including `axum`, `tokio`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `sha2`, `hex`, `argon2`, `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-server-services`, `cms-outbox`.

- [ ] **Step 2: Register in root `Cargo.toml`**
  Add `cms-onboarding = { path = "crates/cms-onboarding" }` to `[workspace.dependencies]` and `"crates/cms-onboarding"` to `[workspace.members]`.

- [ ] **Step 3: Implement `models.rs` and `queries.rs`**
  Define `SetupStatusResponse`, `CompleteSetupRequest`, verification token entities, and SQLx queries for email verification tokens.

- [ ] **Step 4: Implement `service.rs`**
  Implement setup checks, initial admin user creation, secure 32-byte token generation, transactional outbox verification email queuing, and token consumption.

- [ ] **Step 5: Implement `handlers.rs` & `lib.rs`**
  Implement Axum handlers for `GET /api/setup/status`, `POST /api/setup/complete`, `GET /api/auth/verify-email`, `POST /api/auth/resend-verification`. Export `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 6: Write tests in `crates/cms-onboarding/tests/onboarding_test.rs`**
  Test token generation, hash verification, single-use guarantees, and router dispatch.

- [ ] **Step 7: Delegate `crates/cms-api/src/setup/` to `cms-onboarding`**
  Update `cms-api` to route to `cms_onboarding::router(state)`.

- [ ] **Step 8: Verify compilation & commit**
  Run: `cargo test -p cms-onboarding -j 2`
  Git add & commit: `feat(onboarding): extract cms-onboarding vertical slice crate`

---

### Task 4: Extract `crates/cms-reports` Vertical Slice Crate

**Files:**
- Create: `crates/cms-reports/Cargo.toml`
- Create: `crates/cms-reports/src/lib.rs`
- Create: `crates/cms-reports/src/models.rs`
- Create: `crates/cms-reports/src/queries.rs`
- Create: `crates/cms-reports/src/service.rs`
- Create: `crates/cms-reports/src/handlers.rs`
- Create: `crates/cms-reports/tests/reports_test.rs`
- Modify: `crates/cms-api/src/export/mod.rs` (delegate report runs to `cms_reports::router`)
- Modify: `Cargo.toml` (add workspace dependency)

**Interfaces:**
- Consumes: `cms_db`, `cms_error::AppError`, `cms_server_core::app_state::AppState`, `cms_queue`, `cms_storage`, `cms_server_services`
- Produces: `cms_reports::router(state: Arc<AppState>) -> axum::Router`, `cms_reports::ReportService`

- [ ] **Step 1: Create `crates/cms-reports/Cargo.toml`**
  Declare dependencies on `axum`, `tokio`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-server-services`, `cms-queue`, `cms-storage`.

- [ ] **Step 2: Register in root `Cargo.toml`**
  Add `cms-reports = { path = "crates/cms-reports" }` to `[workspace.dependencies]` and `"crates/cms-reports"` to `[workspace.members]`.

- [ ] **Step 3: Implement `models.rs` and `queries.rs`**
  Define `ReportRun`, `CreateReportRunRequest`, `ReportRunResponse`, and queries for report runs lifecycle (`insert`, `update_status`, `get_by_id`, `list_by_workspace`, `delete`).

- [ ] **Step 4: Implement `service.rs`**
  Implement report run lifecycle orchestration, atomic sequence numbering (`cms_db::next_sequence`), job dispatch to `cms_queue`, and storage URL generation.

- [ ] **Step 5: Implement `handlers.rs` & `lib.rs`**
  Implement Axum handlers for `GET /api/app/reports`, `POST /api/app/reports`, `GET /api/app/reports/{id}`, `GET /api/app/reports/{id}/download`, `DELETE /api/app/reports/{id}`. Export `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 6: Write tests in `crates/cms-reports/tests/reports_test.rs`**
  Test report run creation, status transitions, serialization, and router configuration.

- [ ] **Step 7: Wire into `cms-api/src/export/mod.rs`**
  Mount `cms_reports::router(state)` within the export route tree.

- [ ] **Step 8: Verify compilation & commit**
  Run: `cargo test -p cms-reports -j 2`
  Git add & commit: `feat(reports): extract cms-reports vertical slice crate`

---

### Task 5: End-to-End Verification & Documentation Update

**Files:**
- Modify: `AGENTS.md` (update crate diagram and vertical slices documentation)
- Modify: `.superpowers/sdd/progress.md` (track all tasks completion)

- [ ] **Step 1: Run full workspace test suite**
  Run: `cargo test --all-targets -j 2`
  Expected: All tests in all crates pass.

- [ ] **Step 2: Run full Clippy verification**
  Run: `cargo clippy --all-targets --all-features -j 2`
  Expected: 0 warnings, 0 errors.

- [ ] **Step 3: Update `AGENTS.md` and commit**
  Document the new vertical slice crates (`cms-server-services`, `cms-invitations`, `cms-onboarding`, `cms-reports`), their APIs, and update architecture diagrams.
  Git add & commit: `docs: update AGENTS.md with vertical slice crates and shared services architecture`
