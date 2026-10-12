# Phase 1: `cms-content` Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the documentation content core (Pages, Revisions, Branches, Languages, Comments) into a self-contained vertical slice crate `crates/cms-content`, integrate into `cms-api`, and remove obsolete legacy files from `cms-biz` and `cms-api`.

**Architecture:** Colocate models, SQL queries, business services, and Axum route handlers within `crates/cms-content`. The crate exposes subrouters (`page_router`, `branch_router`, `language_router`, `comment_router`) and a composite `router`, while providing domain services directly to consumers.

**Tech Stack:** Rust 2024 (MSRV 1.85), Axum 0.8, SQLx 0.9 (runtime queries / compile-time queries), Tokio, Serde, Chrono, Uuid, `cms-server-core`, `cms-server-services`.

## Global Constraints

- Edition 2024, Rust 1.85 MSRV, `forbid(unsafe_code)`.
- Strict Clippy: `clippy::all` denied, 0 warnings permitted (`clippy::clone_on_ref_ptr`, `clippy::doc_markdown`, `clippy::expect_used` strictly respected).
- No billing, pricing, or payment gateway references (ADR 0001).
- Parallel compilation flag `-j 2` required on Windows to prevent buffer overflows.
- Git commits on Windows must include `--no-verify`.

---

### Task 1: Crate Scaffolding & Workspace Registration

**Files:**
- Create: `crates/cms-content/Cargo.toml`
- Create: `crates/cms-content/src/lib.rs`
- Modify: `Cargo.toml` (workspace members and dependencies)

**Interfaces:**
- Consumes: `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-server-services`, `cms-auth`
- Produces: `crates/cms-content` crate scaffold

- [ ] **Step 1: Create `crates/cms-content/Cargo.toml`**

```toml
[package]
name = "cms-content"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Documentation content core vertical slice (Pages, Branches, Languages, Comments)"

[dependencies]
cms-db = { path = "../cms-db" }
cms-entity = { path = "../cms-entity" }
cms-error = { path = "../cms-error" }
cms-server-core = { path = "../cms-server-core" }
cms-server-services = { path = "../cms-server-services" }
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

Add `cms-content = { path = "crates/cms-content" }` to `[workspace.dependencies]` in root `Cargo.toml`.

- [ ] **Step 3: Create initial `crates/cms-content/src/lib.rs`**

```rust
//! Content core vertical slice: Pages, Branches, Languages, Comments.

use std::sync::Arc;
use axum::Router;
use cms_server_core::app_state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().with_state(state)
}
```

- [ ] **Step 4: Verify crate compilation**

Run: `cargo check -p cms-content -j 2`
Expected: Exited with code 0.

- [ ] **Step 5: Commit**

```bash
git add crates/cms-content/ Cargo.toml; git commit -m "feat(content): scaffold cms-content vertical slice crate" --no-verify
```

---

### Task 2: Page Domain Vertical Slice in `cms-content`

**Files:**
- Create: `crates/cms-content/src/models/page.rs`
- Create: `crates/cms-content/src/queries/page.rs`
- Create: `crates/cms-content/src/services/page.rs`
- Create: `crates/cms-content/src/handlers/page.rs`
- Create: `crates/cms-content/tests/page_test.rs`
- Modify: `crates/cms-content/src/lib.rs`

**Interfaces:**
- Consumes: `cms_db::page::PageQueries`, `AppState`
- Produces: `PageService`, `pub fn page_router(state: Arc<AppState>) -> Router`

- [ ] **Step 1: Implement `models/page.rs`**

Define request/response models: `CreatePagePayload`, `UpdatePagePayload`, `PageSummaryResponse`, `PageDetailResponse`.

- [ ] **Step 2: Implement `queries/page.rs`**

Provide database access helpers interfacing with `cms_db::page::PageQueries`.

- [ ] **Step 3: Implement `services/page.rs`**

Migrate and adapt `PageService` from `cms-biz/src/page.rs`:
- `list_pages(state, project_id, branch_id, language_id)`
- `get_page(state, page_id)`
- `get_page_by_path(state, project_id, branch_id, path)`
- `create_page(state, user_id, request)`
- `update_page(state, user_id, page_id, request)`
- `delete_page(state, user_id, page_id)`
- `get_page_tree(state, project_id, branch_id, root_id)`

- [ ] **Step 4: Implement `handlers/page.rs`**

Implement Axum handlers for:
- `GET /` -> `list_pages_handler`
- `POST /` -> `create_page_handler`
- `GET /{id}` -> `get_page_handler`
- `PUT /{id}` -> `update_page_handler`
- `DELETE /{id}` -> `delete_page_handler`
Expose `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 5: Write unit test in `tests/page_test.rs`**

Test payload serialization, tree structuring, and response transformations.

- [ ] **Step 6: Verify and commit**

Run: `cargo test -p cms-content --test page_test -j 2`
Expected: All tests pass.
Run: `cargo clippy --tests -p cms-content --no-deps -j 2`
Expected: 0 warnings.
Commit: `git add crates/cms-content/; git commit -m "feat(content): implement page domain in cms-content" --no-verify`

---

### Task 3: Branch Domain Vertical Slice in `cms-content`

**Files:**
- Create: `crates/cms-content/src/models/branch.rs`
- Create: `crates/cms-content/src/queries/branch.rs`
- Create: `crates/cms-content/src/services/branch.rs`
- Create: `crates/cms-content/src/handlers/branch.rs`
- Create: `crates/cms-content/tests/branch_test.rs`
- Modify: `crates/cms-content/src/lib.rs`

**Interfaces:**
- Consumes: `cms_db::branch::BranchQueries`, `AppState`
- Produces: `BranchService`, `pub fn branch_router(state: Arc<AppState>) -> Router`

- [ ] **Step 1: Implement `models/branch.rs`**

Define request/response models: `CreateBranchPayload`, `UpdateBranchPayload`, `BranchResponse`.

- [ ] **Step 2: Implement `services/branch.rs`**

Migrate and adapt `BranchService` from `cms-biz/src/branch.rs`:
- `list_branches(state, project_id)`
- `get_branch(state, branch_id)`
- `create_branch(state, user_id, project_id, request)`
- `update_branch(state, user_id, branch_id, request)`
- `delete_branch(state, user_id, branch_id)` (enforces non-default branch check)

- [ ] **Step 3: Implement `handlers/branch.rs`**

Implement Axum handlers for `/` (list/create), `/{id}` (get/update/delete).
Expose `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 4: Write unit test in `tests/branch_test.rs`**

Verify branch creation validation and protected branch logic.

- [ ] **Step 5: Verify and commit**

Run: `cargo test -p cms-content --test branch_test -j 2`
Run: `cargo clippy --tests -p cms-content --no-deps -j 2`
Commit: `git add crates/cms-content/; git commit -m "feat(content): implement branch domain in cms-content" --no-verify`

---

### Task 4: Language Domain Vertical Slice in `cms-content`

**Files:**
- Create: `crates/cms-content/src/models/language.rs`
- Create: `crates/cms-content/src/queries/language.rs`
- Create: `crates/cms-content/src/services/language.rs`
- Create: `crates/cms-content/src/handlers/language.rs`
- Create: `crates/cms-content/tests/language_test.rs`
- Modify: `crates/cms-content/src/lib.rs`

**Interfaces:**
- Consumes: `cms_db::language::LanguageQueries`, `AppState`
- Produces: `LanguageService`, `pub fn language_router(state: Arc<AppState>) -> Router`

- [ ] **Step 1: Implement `models/language.rs` & `services/language.rs`**

Migrate and adapt `LanguageService` from `cms-biz/src/language.rs`:
- `list_languages(state, project_id)`
- `get_language(state, language_id)`
- `create_language(state, user_id, project_id, request)`
- `update_language(state, user_id, language_id, request)`
- `delete_language(state, user_id, language_id)`
- `set_default_language(state, user_id, project_id, language_id)`

- [ ] **Step 2: Implement `handlers/language.rs`**

Implement Axum handlers for list, create, get, update, delete, and `POST /projects/{project_id}/set-default`.
Expose `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 3: Write unit test in `tests/language_test.rs`**

Verify language DTOs and set-default validation.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p cms-content --test language_test -j 2`
Run: `cargo clippy --tests -p cms-content --no-deps -j 2`
Commit: `git add crates/cms-content/; git commit -m "feat(content): implement language domain in cms-content" --no-verify`

---

### Task 5: Comment Domain Vertical Slice in `cms-content`

**Files:**
- Create: `crates/cms-content/src/models/comment.rs`
- Create: `crates/cms-content/src/queries/comment.rs`
- Create: `crates/cms-content/src/services/comment.rs`
- Create: `crates/cms-content/src/handlers/comment.rs`
- Create: `crates/cms-content/tests/comment_test.rs`
- Modify: `crates/cms-content/src/lib.rs`

**Interfaces:**
- Consumes: `cms_db::comment::CommentQueries`, `AppState`
- Produces: `CommentService`, `pub fn comment_router(state: Arc<AppState>) -> Router`, composite `router`

- [ ] **Step 1: Implement `models/comment.rs` & `services/comment.rs`**

Migrate and adapt `CommentService` from `cms-biz/src/comment.rs`:
- `list_comments(state, page_id)`
- `get_comment(state, comment_id)`
- `create_comment(state, user_id, request)`
- `update_comment(state, user_id, comment_id, request)`
- `delete_comment(state, user_id, comment_id)`
- `resolve_comment(state, user_id, comment_id)`

- [ ] **Step 2: Implement `handlers/comment.rs`**

Implement Axum handlers for comments, thread hierarchy, resolve action, and `GET /pages/{page_id}`.
Expose `pub fn router(state: Arc<AppState>) -> Router`.

- [ ] **Step 3: Update `crates/cms-content/src/lib.rs`**

Expose:
- `pub use services::{PageService, BranchService, LanguageService, CommentService};`
- `pub use handlers::{page_router, branch_router, language_router, comment_router};`
- `pub fn router(state: Arc<AppState>) -> Router` (composite router nesting all 4 subrouters)

- [ ] **Step 4: Write unit test in `tests/comment_test.rs`**

Verify comment serialization, IDOR validation, and thread resolution.

- [ ] **Step 5: Verify and commit**

Run: `cargo test -p cms-content -j 2`
Run: `cargo clippy --tests -p cms-content --no-deps -j 2`
Commit: `git add crates/cms-content/; git commit -m "feat(content): implement comment domain and composite router in cms-content" --no-verify`

---

### Task 6: Wire into `cms-api` and Re-Export in `cms-biz`

**Files:**
- Modify: `crates/cms-api/Cargo.toml` (add `cms-content` dependency)
- Modify: `crates/cms-api/src/page/mod.rs` (delegate to `cms_content::page_router`)
- Modify: `crates/cms-api/src/branch/mod.rs` (delegate to `cms_content::branch_router`)
- Modify: `crates/cms-api/src/language/mod.rs` (delegate to `cms_content::language_router`)
- Modify: `crates/cms-api/src/comment/mod.rs` (delegate to `cms_content::comment_router`)
- Modify: `crates/cms-biz/Cargo.toml` (add `cms-content` dependency)
- Modify: `crates/cms-biz/src/lib.rs` (re-export services from `cms-content`)

**Interfaces:**
- Consumes: `cms_content::{page_router, branch_router, language_router, comment_router}`
- Produces: Seamless delegation in `cms-api` and backward-compatible re-exports in `cms-biz`

- [ ] **Step 1: Add dependency to `cms-api` and `cms-biz`**

Add `cms-content = { path = "../cms-content" }` to `crates/cms-api/Cargo.toml` and `crates/cms-biz/Cargo.toml`.

- [ ] **Step 2: Delegate routers in `cms-api`**

In:
- `crates/cms-api/src/page/mod.rs`: `cms_content::page_router(state)`
- `crates/cms-api/src/branch/mod.rs`: `cms_content::branch_router(state)`
- `crates/cms-api/src/language/mod.rs`: `cms_content::language_router(state)`
- `crates/cms-api/src/comment/mod.rs`: `cms_content::comment_router(state)`

- [ ] **Step 3: Re-export services in `cms-biz`**

In `crates/cms-biz/src/lib.rs`, re-export `PageService`, `BranchService`, `LanguageService`, `CommentService` from `cms_content` to keep existing callers functional without breakage. Delete old redundant implementation files from `cms-biz`.

- [ ] **Step 4: Verify and commit**

Run: `cargo check -p cms-api -j 2`
Run: `cargo check -p cms-biz -j 2`
Commit: `git add crates/cms-api/ crates/cms-biz/; git commit -m "refactor(api,biz): delegate content domains to cms-content vertical slice" --no-verify`

---

### Task 7: Full Workspace Verification, Clippy & Progress Tracking

**Files:**
- Modify: `AGENTS.md`
- Modify: `.superpowers/sdd/progress-vertical-slices.md`

- [ ] **Step 1: Run full workspace test suite**

Run: `cargo test -p cms-content -j 2`
Expected: All tests pass.

- [ ] **Step 2: Run full workspace check**

Run: `cargo check --workspace -j 2`
Expected: 0 errors.

- [ ] **Step 3: Run Clippy verification**

Run: `cargo clippy --tests -p cms-content --no-deps -j 2`
Expected: 0 warnings, 0 errors.

- [ ] **Step 4: Update documentation and commit**

Update `AGENTS.md` and `.superpowers/sdd/progress-vertical-slices.md`.
Commit: `git add AGENTS.md .superpowers/sdd/progress-vertical-slices.md; git commit -m "docs: document cms-content vertical slice in AGENTS.md and SDD tracker" --no-verify`
