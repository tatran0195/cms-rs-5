# True Vertical Slice Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform `cms-rs-4` from a horizontal layered monolith disguised as crates into a true Vertical Slice Architecture where each crate is autonomous, owns its database models & SQL queries, eliminates all shims and duplicated auth extractors, and communicates via a clean Directed Acyclic Graph (DAG).

**Architecture:** 
- `cms-db` is demoted strictly to database infrastructure (pool, migrations, RLS tenant context, atomic sequences, audit hash-chains).
- `cms-entity` is dissolved; universal pagination primitives move to `cms-server-core::pagination`.
- Each vertical slice owns its domain aggregate modules (models, SQLx queries, business logic, handlers, router).
- `cms-auth` exports the single, canonical `AuthUser` extractor; all 7 duplicated extractors (`ContentUser`, `ProjectUser`, etc.) are deleted.

**Tech Stack:** Rust (Edition 2024, MSRV 1.85), Axum 0.8, SQLx 0.9 (offline compile-time verified), Tokio, Tower, Zanzibar ReBAC (`cms-authz`).

## Global Constraints

- **No SaaS Plan/Billing or Marketing Pages:** Dedicated internal documentation platform only (ADR 0001).
- **Strict Rust Standards:** Edition 2024, `forbid(unsafe_code)`, `unwrap_used` / `expect_used` / `panic` denied.
- **Strict Clippy:** `clippy::all` denied, `clippy::pedantic` and `clippy::nursery` warned.
- **SQLx Compile-Time Checks:** All queries must be compile-time checked with `SQLX_OFFLINE=true`.
- **Zero Dead Code & Zero Shims:** No empty re-export folders (`pub use cms_db::...`), no unused functions or variables.

---

### Task 1: Foundation & Canonical Authentication (`cms-server-core` & `cms-auth`)

**Files:**
- Create: `crates/cms-server-core/src/pagination.rs`
- Modify: `crates/cms-server-core/src/lib.rs`
- Modify: `crates/cms-auth/src/middleware.rs`
- Modify: `crates/cms-auth/src/lib.rs`
- Test: `crates/cms-auth/tests/` or in-crate test module in `crates/cms-auth/src/middleware.rs`

**Interfaces:**
- Consumes: `cms_server_core::app_state::AppState`, `cms_error::AppError`
- Produces: `cms_server_core::pagination::PaginatedResponse<T>`, `cms_auth::AuthUser`

- [ ] **Step 1: Move `PaginatedResponse` into `cms-server-core::pagination`**

Ensure `PaginatedResponse<T>` is available from `cms_server_core::pagination` so slices do not depend on `cms-entity::common`:
```rust
// crates/cms-server-core/src/pagination.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let page_size = page_size.max(1);
        let total_pages = (total + page_size - 1) / page_size;
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}
```
Export it from `crates/cms-server-core/src/lib.rs`.

- [ ] **Step 2: Implement and export canonical `AuthUser` in `cms-auth`**

In `crates/cms-auth/src/middleware.rs`:
```rust
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use cms_error::AppError;
use uuid::Uuid;
use crate::models::{Claims, UserResponse};

/// Canonical authenticated user extractor for all vertical slices.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: String,
    pub claims: Claims,
    pub user: UserResponse,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_extractor = AuthExtractor::from_request_parts(parts, state).await?;
        let user_id = Uuid::parse_str(&auth_extractor.user.id)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(Self {
            id: user_id,
            role: auth_extractor.claims.role.clone(),
            claims: auth_extractor.claims,
            user: auth_extractor.user,
        })
    }
}
```
Export `pub use middleware::AuthUser;` from `crates/cms-auth/src/lib.rs`.

- [ ] **Step 3: Verify compilation of `cms-server-core` and `cms-auth`**

Run: `SQLX_OFFLINE=true cargo check -p cms-server-core -p cms-auth`
Expected: PASS with 0 errors.

- [ ] **Step 4: Commit Foundation & Auth changes**

```bash
git add crates/cms-server-core crates/cms-auth
git commit -m "feat(auth,core): add canonical AuthUser extractor and PaginatedResponse"
```

---

### Task 2: Sovereign Projects Slice (`cms-projects`)

**Files:**
- Modify: `crates/cms-projects/Cargo.toml`
- Modify: `crates/cms-projects/src/lib.rs`
- Delete: `crates/cms-projects/src/handlers/auth.rs` (duplicate `ProjectUser`)
- Modify: `crates/cms-projects/src/handlers/mod.rs`
- Consolidate: `crates/cms-projects/src/queries/` and `models/` to own all project, domain, and reader access data
- Delete from `cms-db`: `crates/cms-db/src/project.rs`, `crates/cms-db/src/domain.rs`, `crates/cms-db/src/reader_access.rs`

**Interfaces:**
- Consumes: `cms_auth::AuthUser`, `cms_server_core::pagination::PaginatedResponse`, `cms_db::PgPool`
- Produces: `cms_projects::{Project, ProjectQueries, ProjectMember, DomainQueries, project_router}`

- [ ] **Step 1: Replace `ProjectUser` with `cms_auth::AuthUser` in `cms-projects`**

Delete `crates/cms-projects/src/handlers/auth.rs`.
In `crates/cms-projects/src/handlers/*.rs`, replace all occurrences of `ProjectUser` with `cms_auth::AuthUser`.

- [ ] **Step 2: Ensure `cms-projects` owns all its SQL queries and models**

Verify `crates/cms-projects/src/queries/project.rs`, `member.rs`, `domain.rs`, and `reader_access.rs` have zero dependencies on `cms_db::project`, `cms_db::domain`, or `cms_db::reader_access`.
Ensure models in `crates/cms-projects/src/models/` are self-contained.

- [ ] **Step 3: Remove duplicate query implementations from `cms-db`**

Remove `pub mod project;`, `pub mod domain;`, `pub mod reader_access;` from `crates/cms-db/src/lib.rs`.
Delete `crates/cms-db/src/project.rs`, `crates/cms-db/src/domain.rs`, and `crates/cms-db/src/reader_access.rs`.
Any external crate needing project queries must import `cms_projects::ProjectQueries`.

- [ ] **Step 4: Verify `cms-projects` compiles cleanly**

Run: `SQLX_OFFLINE=true cargo check -p cms-projects`
Expected: PASS with 0 errors.

- [ ] **Step 5: Commit `cms-projects` sovereignty**

```bash
git add crates/cms-projects crates/cms-db
git commit -m "refactor(projects): make cms-projects sovereign, delete duplicate db queries and ProjectUser"
```

---

### Task 3: Sovereign Content Slice (`cms-content`)

**Files:**
- Modify: `crates/cms-content/Cargo.toml` (add `cms-projects` dependency)
- Modify: `crates/cms-content/src/lib.rs`
- Delete: `crates/cms-content/src/handlers/auth.rs` (duplicate `ContentUser`)
- Delete: `crates/cms-content/src/queries/` shims (`pub use cms_db::...`)
- Delete: `crates/cms-content/src/models/` shims (`pub use cms_entity::...`)
- Move queries from `cms-db` into `cms-content/src/`:
  - `crates/cms-db/src/page.rs` -> `crates/cms-content/src/queries/page.rs` (or merged into `page.rs`)
  - `crates/cms-db/src/branch.rs` -> `crates/cms-content/src/queries/branch.rs`
  - `crates/cms-db/src/language.rs` -> `crates/cms-content/src/queries/language.rs`
  - `crates/cms-db/src/comment.rs` -> `crates/cms-content/src/queries/comment.rs`
- Delete: `crates/cms-db/src/page.rs`, `branch.rs`, `language.rs`, `comment.rs`

**Interfaces:**
- Consumes: `cms_auth::AuthUser`, `cms_projects::ProjectQueries`, `cms_server_core::pagination::PaginatedResponse`
- Produces: `cms_content::{Page, PageQueries, Branch, BranchQueries, Language, Comment, page_router, branch_router, ...}`

- [ ] **Step 1: Add `cms-projects` dependency to `crates/cms-content/Cargo.toml`**

Add:
```toml
cms-projects = { workspace = true }
```

- [ ] **Step 2: Replace `ContentUser` with `cms_auth::AuthUser`**

Delete `crates/cms-content/src/handlers/auth.rs`.
In `crates/cms-content/src/handlers/*.rs`, replace all `ContentUser` with `cms_auth::AuthUser`.

- [ ] **Step 3: Move models and SQLx queries into `cms-content`**

Move the actual SQLx query implementations from `cms-db` (`page.rs`, `branch.rs`, `language.rs`, `comment.rs`) into `cms-content`.
Remove the empty shims in `cms-content/src/queries/` and `cms-content/src/models/`.
Update `cms-content/src/services/page.rs` to call `cms_projects::ProjectQueries` instead of `cms_db::project::ProjectQueries`.

- [ ] **Step 4: Remove `page.rs`, `branch.rs`, `language.rs`, `comment.rs` from `cms-db`**

Remove their declarations from `crates/cms-db/src/lib.rs` and delete the files.

- [ ] **Step 5: Verify `cms-content` compiles cleanly**

Run: `SQLX_OFFLINE=true cargo check -p cms-content`
Expected: PASS with 0 errors.

- [ ] **Step 6: Commit `cms-content` sovereignty**

```bash
git add crates/cms-content crates/cms-db
git commit -m "refactor(content): make cms-content sovereign, eliminate shims and ContentUser"
```

---

### Task 4: Migrate Deployments, Git, and Sites Slices

**Files:**
- Modify: `crates/cms-deployments/Cargo.toml`, `src/lib.rs`, `src/auth.rs`
- Modify: `crates/cms-git/Cargo.toml`, `src/lib.rs`, `src/auth.rs`
- Modify: `crates/cms-sites/Cargo.toml`, `src/lib.rs`
- Delete: `crates/cms-db/src/deployment.rs`, `crates/cms-db/src/git.rs`

**Interfaces:**
- Consumes: `cms_auth::AuthUser`, `cms_projects`, `cms_content`
- Produces: `cms_deployments::router`, `cms_git::router`, `cms_sites`

- [ ] **Step 1: Migrate `cms-deployments`**

- Move deployment queries from `crates/cms-db/src/deployment.rs` into `crates/cms-deployments`.
- Delete `crates/cms-deployments/src/auth.rs` (`DeploymentUser`), use `cms_auth::AuthUser`.
- Import `Project` and `Page` from `cms_projects` and `cms_content`.
- Delete `crates/cms-db/src/deployment.rs`.

- [ ] **Step 2: Migrate `cms-git`**

- Move git queries from `crates/cms-db/src/git.rs` into `crates/cms-git`.
- Delete `crates/cms-git/src/auth.rs` (`GitUser`), use `cms_auth::AuthUser`.
- Import from `cms_projects` and `cms_content`.
- Delete `crates/cms-db/src/git.rs`.

- [ ] **Step 3: Update `cms-sites`**

- Point any project or content queries to `cms_projects` and `cms_content`.

- [ ] **Step 4: Verify compilation**

Run: `SQLX_OFFLINE=true cargo check -p cms-deployments -p cms-git -p cms-sites`
Expected: PASS with 0 errors.

- [ ] **Step 5: Commit Deployments, Git, and Sites**

```bash
git add crates/cms-deployments crates/cms-git crates/cms-sites crates/cms-db
git commit -m "refactor: make cms-deployments and cms-git sovereign, remove duplicate extractors"
```

---

### Task 5: Migrate Remaining Slices & Purge `cms-db`

**Files:**
- Modify: `crates/cms-assets/`, `crates/cms-themes/`, `crates/cms-integrations/`
- Modify: `crates/cms-notifications/`, `crates/cms-invitations/`, `crates/cms-onboarding/`, `crates/cms-reports/`
- Modify: `crates/cms-admin/`, `crates/cms-analytics/`, `crates/cms-api-specs/`
- Clean: `crates/cms-db/src/lib.rs`

**Interfaces:**
- Consumes: `cms_auth::AuthUser`, `cms_db::PgPool`
- Produces: Sovereign domain slices with zero domain queries remaining in `cms-db`.

- [ ] **Step 1: Migrate `cms-assets`, `cms-themes`, and `cms-integrations`**

- Delete `AssetUser`, `ThemeUser`, `IntegrationUser`; import `cms_auth::AuthUser`.
- Move queries from `cms-db/src/asset.rs`, `theme.rs`, `integration.rs` into the slices.
- Delete those files from `cms-db`.

- [ ] **Step 2: Migrate `cms-notifications`, `cms-invitations`, `cms-onboarding`, `cms-reports`**

- Move remaining queries (`export.rs`, `notification.rs`, `setup.rs`) into their slices.
- Delete those files from `cms-db`.

- [ ] **Step 3: Migrate `cms-admin`, `cms-analytics`, `cms-api-specs`, `cms-search`**

- Move `platform_event.rs`, `analytics.rs`, `openapi.rs`, `search_index.rs` into their slices.
- Delete those files from `cms-db`.

- [ ] **Step 4: Purge `cms-db` to pure infrastructure**

Ensure `crates/cms-db/src/lib.rs` only exposes:
```rust
pub mod pool;
pub mod context;
pub mod sequence;
pub mod audit;
```
Plus migration runner and health check. Zero domain models or queries remain.

- [ ] **Step 5: Verify workspace compiles**

Run: `SQLX_OFFLINE=true cargo check --workspace`
Expected: PASS with 0 errors.

- [ ] **Step 6: Commit remaining slices migration and `cms-db` purge**

```bash
git add crates/
git commit -m "refactor: complete vertical slice migration across all secondary crates and purge cms-db"
```

---

### Task 6: Deprecate `cms-entity`, Composition Root Assembly & Final Verification

**Files:**
- Modify: `apps/api/router.rs`, `apps/api/main.rs`
- Modify: Root `Cargo.toml`
- Test: Full integration test suite `cargo test --workspace`

- [ ] **Step 1: Dissolve `cms-entity`**

Remove `cms-entity` from `Cargo.toml` members and dependencies once all domain models are owned by their respective vertical slices.

- [ ] **Step 2: Verify Composition Root (`apps/api`)**

Ensure `apps/api/router.rs` mounts all vertical slice routers cleanly and applies middleware.
Verify `apps/api/main.rs` builds with zero warnings.

- [ ] **Step 3: Run comprehensive verification suite**

Run: `SQLX_OFFLINE=true cargo check --workspace`
Expected: 0 errors, 0 clippy warnings.

Run: `cargo test --workspace --no-run`
Expected: 0 build errors for tests.

Run: `cargo test -p cms-auth -p cms-authz`
Expected: All unit and ReBAC integration tests PASS.

- [ ] **Step 4: Commit Composition Root & final cleanup**

```bash
git add apps/api Cargo.toml Cargo.lock
git commit -m "chore: dissolve cms-entity, update composition root, and complete vertical slice architecture"
```
