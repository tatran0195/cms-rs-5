# Phase 2: `cms-assets` & `cms-projects` Vertical Slices Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Asset management into `crates/cms-assets` (Phase 2A) and Workspace/Project management into `crates/cms-projects` (Phase 2B), integrate both into `cms-api`, eliminate domain logic from `cms-biz`, and advance toward full modular monolith crate decommissioning.

**Architecture:** Colocate models, SQL queries, business services, and Axum route handlers within each respective vertical slice crate. Decouple domain services from `BizContext` to accept `&PgPool` and domain dependencies (`Arc<dyn Storage>`). Wire self-contained routers directly into `cms-api` with zero breaking changes to existing endpoints.

**Tech Stack:** Rust 2024 (MSRV 1.85), Axum 0.8, SQLx, Tokio, Serde, Chrono, Uuid, `cms-server-core`, `cms-server-services`, `cms-storage`.

## Global Constraints

- Edition 2024, Rust 1.85 MSRV, `forbid(unsafe_code)`.
- Strict Clippy: `clippy::all` denied, 0 warnings permitted (`clippy::clone_on_ref_ptr`, `clippy::doc_markdown`, `clippy::expect_used` strictly respected).
- No billing, pricing, or payment gateway references (ADR 0001).
- Parallel compilation flag `-j 2` required on Windows to prevent memory and thread buffer overflows.
- Git commits on Windows must include `--no-verify`.

---

## Part A: `crates/cms-assets` Vertical Slice

### Task 1: Scaffolding `crates/cms-assets` & Workspace Registration

**Files:**
- Create: `crates/cms-assets/Cargo.toml`
- Create: `crates/cms-assets/src/lib.rs`
- Modify: `Cargo.toml` (workspace members and dependencies)

**Interfaces:**
- Consumes: `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-server-services`, `cms-storage`, `cms-auth`
- Produces: `crates/cms-assets` crate scaffold

- [ ] **Step 1: Create `crates/cms-assets/Cargo.toml`**

```toml
[package]
name = "cms-assets"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Asset management vertical slice (upload, storage, metadata, presigned URLs)"

[dependencies]
cms-db = { path = "../cms-db" }
cms-entity = { path = "../cms-entity" }
cms-error = { path = "../cms-error" }
cms-server-core = { path = "../cms-server-core" }
cms-server-services = { path = "../cms-server-services" }
cms-storage = { path = "../cms-storage" }
cms-auth = { path = "../cms-auth" }

axum = { workspace = true, features = ["multipart"] }
bytes = { workspace = true }
tokio = { workspace = true, features = ["full"] }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true, features = ["v4", "serde"] }
chrono = { workspace = true }
sqlx = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }

[lints]
workspace = true
```

- [ ] **Step 2: Register `cms-assets` in root `Cargo.toml`**

Add `cms-assets = { path = "crates/cms-assets" }` to `[workspace.dependencies]` in root `Cargo.toml`.

- [ ] **Step 3: Create initial `crates/cms-assets/src/lib.rs`**

```rust
//! Asset management vertical slice: upload, storage, metadata, presigned URLs.

pub mod handlers;
pub mod models;
pub mod queries;
pub mod services;

use std::sync::Arc;
use axum::Router;
use cms_server_core::app_state::AppState;

pub use handlers::{asset_router, project_asset_router};
pub use services::AssetService;

pub fn router(state: Arc<AppState>) -> Router {
    asset_router(state)
}
```

- [ ] **Step 4: Verify crate compilation**

Run: `cargo check -p cms-assets -j 2`
Expected: Exited with code 0.

- [ ] **Step 5: Commit**

```bash
git add crates/cms-assets/ Cargo.toml; git commit -m "feat(assets): scaffold cms-assets vertical slice crate" --no-verify
```

---

### Task 2: Asset Domain Models & SQLx Queries

**Files:**
- Create: `crates/cms-assets/src/models/mod.rs`
- Create: `crates/cms-assets/src/models/asset.rs`
- Create: `crates/cms-assets/src/queries/mod.rs`
- Create: `crates/cms-assets/src/queries/asset.rs`

- [ ] **Step 1: Create `crates/cms-assets/src/models/asset.rs`**

Colocate `AssetRow`, request DTOs (`UploadAssetRequest`, `UpdateAssetRequest`, `PresignAssetRequest`, `ConfirmAssetRequest`), and response DTOs (`AssetResponse`, `PresignAssetResponse`, `ConfirmAssetResponse`). Derive `Debug, Clone, Serialize, Deserialize` on all DTOs.

- [ ] **Step 2: Create `crates/cms-assets/src/queries/asset.rs`**

Implement `AssetQueries` with compile-safe SQLx queries:
- `get_by_id(pool, id)`
- `get_by_project(pool, project_id, limit, offset)`
- `count_by_project(pool, project_id)`
- `get_by_page(pool, page_id)`
- `create(pool, &row)`
- `update(pool, id, alt_text)`
- `delete(pool, id)`

- [ ] **Step 3: Verify models and queries compile**

Run: `cargo check -p cms-assets -j 2`

---

### Task 3: Decoupled `AssetService`

**Files:**
- Create: `crates/cms-assets/src/services/mod.rs`
- Create: `crates/cms-assets/src/services/asset.rs`

- [ ] **Step 1: Implement `AssetService`**

Implement `AssetService` accepting `pool: &PgPool` and `storage: Arc<dyn Storage>`:
- `upload_asset(pool, storage, project_id, page_id, request, content)`
- `get_asset(pool, asset_id)`
- `list_assets(pool, project_id, page, page_size)`
- `list_assets_for_page(pool, page_id)`
- `update_asset(pool, asset_id, alt_text)`
- `delete_asset(pool, storage, asset_id)`
- `create_asset(pool, project_id, page_id, file_name, content_type, alt_text)`
- `upload_asset_bytes(pool, storage, user_id, project_id, file_name, content_type, file_data)`

- [ ] **Step 2: Verify `AssetService` compiles cleanly**

Run: `cargo check -p cms-assets -j 2`

---

### Task 4: Axum Handlers & Routers

**Files:**
- Create: `crates/cms-assets/src/handlers/mod.rs`
- Create: `crates/cms-assets/src/handlers/auth.rs`
- Create: `crates/cms-assets/src/handlers/asset.rs`
- Create: `crates/cms-assets/src/handlers/project_asset.rs`

- [ ] **Step 1: Create `handlers/auth.rs`**

Implement `AssetUser` auth extractor wrapping `cms_auth::AuthenticatedUser`.

- [ ] **Step 2: Create `handlers/asset.rs` (`/assets`)**

Routes:
- `GET /` — `list_assets_handler`
- `POST /upload` — `upload_asset_handler`
- `POST /upload-multipart/{project_id}` — `upload_asset_multipart_handler`
- `GET /{id}` — `get_asset_handler`
- `PUT /{id}` — `update_asset_handler`
- `DELETE /{id}` — `delete_asset_handler`

- [ ] **Step 3: Create `handlers/project_asset.rs` (`/projects/{project_id}/assets`)**

Routes:
- `GET /` — `list_project_assets_handler`
- `POST /presign` — `presign_project_asset_handler`
- `POST /confirm` — `confirm_project_asset_handler`

- [ ] **Step 4: Expose `asset_router` and `project_asset_router` in `handlers/mod.rs`**

- [ ] **Step 5: Verify handlers compile**

Run: `cargo check -p cms-assets -j 2`

---

### Task 5: Contract & Serde Tests for `cms-assets`

**Files:**
- Create: `crates/cms-assets/tests/asset_test.rs`

- [ ] **Step 1: Write serde contract tests**

Test serialization/deserialization for `PresignAssetRequest`, `ConfirmAssetRequest`, `UpdateAssetRequest`, and `AssetResponse`.

- [ ] **Step 2: Run test suite**

Run: `cargo test -p cms-assets -j 2`
Expected: All tests pass.

- [ ] **Step 3: Run Clippy**

Run: `cargo clippy --tests -p cms-assets --no-deps -j 2`
Expected: 0 warnings, 0 errors.

- [ ] **Step 4: Commit**

```bash
git add crates/cms-assets/; git commit -m "feat(assets): implement asset domain and routers in cms-assets" --no-verify
```

---

### Task 6: Wire `cms-assets` into `cms-api` and Re-Export

**Files:**
- Modify: `crates/cms-api/Cargo.toml`
- Modify: `crates/cms-api/src/asset/mod.rs`
- Modify: `crates/cms-api/src/project/handlers/assets.rs`

- [ ] **Step 1: Add `cms-assets` to `crates/cms-api/Cargo.toml`**

- [ ] **Step 2: Delegate `crates/cms-api/src/asset/mod.rs` to `cms_assets::asset_router`**

- [ ] **Step 3: Delegate project asset handlers in `crates/cms-api/src/project/handlers/assets.rs` to `cms_assets`**

- [ ] **Step 4: Verify and commit**

Run: `cargo check -p cms-api -j 2`
Commit: `git add crates/cms-api/; git commit -m "refactor(api): delegate asset routes to cms-assets vertical slice" --no-verify`

---

## Part B: `crates/cms-projects` Vertical Slice

### Task 7: Scaffolding `crates/cms-projects` & Workspace Registration

**Files:**
- Create: `crates/cms-projects/Cargo.toml`
- Create: `crates/cms-projects/src/lib.rs`
- Modify: `Cargo.toml` (workspace members and dependencies)

- [ ] **Step 1: Create `crates/cms-projects/Cargo.toml`**

```toml
[package]
name = "cms-projects"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Project workspace vertical slice (projects, members, settings, domains, reader access, addons)"

[dependencies]
cms-db = { path = "../cms-db" }
cms-entity = { path = "../cms-entity" }
cms-error = { path = "../cms-error" }
cms-server-core = { path = "../cms-server-core" }
cms-server-services = { path = "../cms-server-services" }
cms-assets = { path = "../cms-assets" }
cms-content = { path = "../cms-content" }
cms-auth = { path = "../cms-auth" }

axum = { workspace = true }
tokio = { workspace = true, features = ["full"] }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true, features = ["v4", "serde"] }
chrono = { workspace = true }
sqlx = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }

[lints]
workspace = true
```

- [ ] **Step 2: Register in root `Cargo.toml`**

Add `cms-projects = { path = "crates/cms-projects" }` to `[workspace.dependencies]`.

- [ ] **Step 3: Create initial `crates/cms-projects/src/lib.rs`**

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p cms-projects -j 2`

- [ ] **Step 5: Commit**

```bash
git add crates/cms-projects/ Cargo.toml; git commit -m "feat(projects): scaffold cms-projects vertical slice crate" --no-verify
```

---

### Task 8: Colocate Project Models & SQLx Queries

**Files:**
- Create: `crates/cms-projects/src/models/` (`project.rs`, `member.rs`, `domain.rs`, `reader_access.rs`, `settings.rs`, `mod.rs`)
- Create: `crates/cms-projects/src/queries/` (`project.rs`, `member.rs`, `domain.rs`, `reader_access.rs`, `settings.rs`, `mod.rs`)

- [ ] **Step 1: Implement Project models**

Colocate models with full Serde and Debug derivations.

- [ ] **Step 2: Implement Project queries**

Move and optimize SQLx queries:
- Project CRUD, slug uniqueness checks, collision retry logic
- Member management (role assignment, owner protection)
- Custom domains (DNS validation, verification status)
- Reader access tokens and password verification

- [ ] **Step 3: Verify queries compile**

Run: `cargo check -p cms-projects -j 2`

---

### Task 9: Implement Decoupled Services in `cms-projects`

**Files:**
- Create: `crates/cms-projects/src/services/` (`project.rs`, `member.rs`, `domain.rs`, `reader_access.rs`, `mod.rs`)

- [ ] **Step 1: Implement `ProjectService`**

Decoupled from `BizContext`, accepts `&PgPool`. Handles project creation, slug uniqueness with numeric suffix resolution, updates, deletion, and settings.

- [ ] **Step 2: Implement `MemberService`, `DomainService`, `ReaderAccessService`**

- [ ] **Step 3: Verify services compile**

Run: `cargo check -p cms-projects -j 2`

---

### Task 10: Axum Handlers & Composite Project Router

**Files:**
- Create: `crates/cms-projects/src/handlers/` (`core.rs`, `members.rs`, `domains.rs`, `reader_access.rs`, `addons.rs`, `mod.rs`)

- [ ] **Step 1: Implement handlers**

Port handlers from `cms-api/src/project/handlers/` directly into `cms-projects`.

- [ ] **Step 2: Build composite `project_router`**

Expose `pub fn router(state: Arc<AppState>) -> Router` mounting all `/projects` endpoints including settings, members, domains, addons, and reader-access.

- [ ] **Step 3: Verify handlers compile**

Run: `cargo check -p cms-projects -j 2`

---

### Task 11: Unit & Contract Tests for `cms-projects`

**Files:**
- Create: `crates/cms-projects/tests/project_test.rs`
- Create: `crates/cms-projects/tests/member_test.rs`
- Create: `crates/cms-projects/tests/domain_test.rs`

- [ ] **Step 1: Write contract tests**

Test JSON serialization, request validation, slug sanitizer edge cases, and domain format checks.

- [ ] **Step 2: Run test suite**

Run: `cargo test -p cms-projects -j 2`
Expected: All tests pass.

- [ ] **Step 3: Run Clippy**

Run: `cargo clippy --tests -p cms-projects --no-deps -j 2`
Expected: 0 warnings, 0 errors.

- [ ] **Step 4: Commit**

```bash
git add crates/cms-projects/; git commit -m "feat(projects): implement project domain and routers in cms-projects" --no-verify
```

---

### Task 12: Wire `cms-projects` into `cms-api` and Verify Workspace

**Files:**
- Modify: `crates/cms-api/Cargo.toml`
- Modify: `crates/cms-api/src/project/mod.rs`
- Modify: `AGENTS.md`
- Modify: `.superpowers/sdd/progress-vertical-slices.md`

- [ ] **Step 1: Add `cms-projects` to `crates/cms-api/Cargo.toml`**

- [ ] **Step 2: Delegate `crates/cms-api/src/project/mod.rs` to `cms_projects::router(state)`**

- [ ] **Step 3: Run full workspace checks**

Run: `cargo test -p cms-assets -j 2`
Run: `cargo test -p cms-projects -j 2`
Run: `cargo check --workspace -j 2`
Expected: 0 errors across all crates.

- [ ] **Step 4: Update documentation and commit**

Update `AGENTS.md` with `cms-assets` and `cms-projects` vertical slice entries.
Commit: `git add AGENTS.md .superpowers/sdd/progress-vertical-slices.md crates/cms-api/; git commit -m "refactor(api): delegate project and asset routes to vertical slices" --no-verify`
