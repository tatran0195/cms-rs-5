# Crate Consolidation & Infrastructure Unification (`cms-infra`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate 7 infrastructure crates (`cms-config`, `cms-storage`, `cms-search`, `cms-queue`, `cms-mailer`, `cms-analytics`, `cms-i18n`) into a single `crates/cms-infra` crate, eliminate the `cms-middleware` facade in favor of `cms-server-core`, clean up legacy crate artifacts, and ensure full workspace compilation and tests.

**Architecture:** A high-cohesion `crates/cms-infra` crate exposing submodules (`cms_infra::config`, `cms_infra::storage`, `cms_infra::search`, `cms_infra::queue`, `cms_infra::mailer`, `cms_infra::analytics`, `cms_infra::i18n`) with feature flags for optional heavy dependencies (`s3`, `vector`, `memory`). Consumers (`cms-server-core`, `cms-server-services`, vertical slices, `apps/api`) consume `cms-infra` directly.

**Tech Stack:** Rust Edition 2024 (MSRV 1.85), Axum 0.8, SQLx 0.9, Tokio 1, Tantivy 0.26, Lindera 6, FastEmbed 7, Lettre 0.11, MiniJinja 3, Cargo workspace.

## Global Constraints
- Rust Edition 2024, MSRV 1.85, with `forbid(unsafe_code)`
- Strict Clippy denials: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, `wildcard_imports`
- Zero placeholders: All implementations, re-exports, and tests must be 100% complete and compiling
- Backwards compatibility: Existing behavior, configuration parsing, storage, search, queue, and mailer contracts are fully preserved

---

### Task 1: Scaffold `crates/cms-infra` and Migrate Config, i18n, Analytics

**Files:**
- Create: `crates/cms-infra/Cargo.toml`
- Create: `crates/cms-infra/build.rs`
- Create: `crates/cms-infra/src/lib.rs`
- Create: `crates/cms-infra/src/config/mod.rs` and copy config submodule files
- Create: `crates/cms-infra/src/i18n/mod.rs` and copy i18n submodule files
- Create: `crates/cms-infra/src/analytics/mod.rs` and copy analytics submodule files
- Test: `crates/cms-infra/tests/config_test.rs`

**Interfaces:**
- Produces:
  - `cms_infra::config::{Config, DatabaseConfig, StorageConfig, SearchConfig, QueueConfig, MailerConfig, AnalyticsConfig, ...}`
  - `cms_infra::i18n::{Locale, Catalog, translate, accept_language_tags, ...}`
  - `cms_infra::analytics::{AnalyticsStore, PostgresAnalyticsStore, AnalyticsService, create_analytics_store}`

- [x] **Step 1: Create `crates/cms-infra/Cargo.toml`**

```toml
[package]
name = "cms-infra"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Consolidated infrastructure layer for CMS: config, storage, search, queue, mailer, analytics, i18n"
build = "build.rs"

[dependencies]
# Internal workspace crates
cms-entity = { path = "../cms-entity" }
cms-db = { path = "../cms-db" }
cms-error = { path = "../cms-error" }

# Configuration
serde = { workspace = true }
serde_json = { workspace = true }
config = { workspace = true }
figment = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
tokio = { workspace = true, features = ["full"] }
async-trait = { workspace = true }
bytes = { workspace = true }
sqlx = { workspace = true }
rusqlite = { workspace = true }

# Search dependencies
lindera = { workspace = true }
unicode-normalization = { workspace = true }
tantivy = { workspace = true }
pulldown-cmark = { workspace = true }
parking_lot = { workspace = true }
fastembed = { workspace = true, optional = true }
reqwest = { workspace = true, optional = true }

# Mailer dependencies
lettre = { workspace = true }
minijinja = { workspace = true }

# Storage dependencies
aws-sdk-s3 = { workspace = true, optional = true }
aws-config = { workspace = true, optional = true }
aws-credential-types = { workspace = true, optional = true }

[build-dependencies]
serde_json = { workspace = true }

[features]
default = ["memory", "vector"]
memory = []
vector = ["dep:fastembed", "dep:reqwest"]
s3 = ["dep:aws-sdk-s3", "dep:aws-config", "dep:aws-credential-types"]

[dev-dependencies]
tempfile = { workspace = true }

[lints]
workspace = true
```

- [x] **Step 2: Create `crates/cms-infra/build.rs` to embed translations and email messages**

Create `crates/cms-infra/build.rs` processing `../../packages/i18n/messages` for both generic i18n (`api_error_`, `email_`) and mailer namespaces into `embedded_messages.rs` in `OUT_DIR`.

- [x] **Step 3: Copy and organize `config`, `i18n`, and `analytics` into `crates/cms-infra/src/`**
- Move `crates/cms-config/src/*` into `crates/cms-infra/src/config/`
- Move `crates/cms-i18n/src/*` into `crates/cms-infra/src/i18n/`
- Move `crates/cms-analytics/src/*` into `crates/cms-infra/src/analytics/`
- Wire them up in `crates/cms-infra/src/lib.rs`:
```rust
#![forbid(unsafe_code)]

pub mod analytics;
pub mod config;
pub mod i18n;
```
- Update internal module references (e.g. `cms_config::` to `crate::config::`).

- [x] **Step 4: Check compilation of `cms-infra`**

Run: `cargo check -p cms-infra`
Expected: PASS

- [x] **Step 5: Commit Task 1**

Run:
```bash
git add crates/cms-infra
git commit -m "feat(infra): scaffold cms-infra crate with config, i18n, and analytics modules"
```

---

### Task 2: Migrate Storage, Queue, Mailer, and Search into `crates/cms-infra`

**Files:**
- Create: `crates/cms-infra/src/storage/` (from `crates/cms-storage/src/`)
- Create: `crates/cms-infra/src/queue/` (from `crates/cms-queue/src/`)
- Create: `crates/cms-infra/src/mailer/` (from `crates/cms-mailer/src/`)
- Create: `crates/cms-infra/src/search/` (from `crates/cms-search/src/`)
- Modify: `crates/cms-infra/src/lib.rs`
- Test: `crates/cms-infra/tests/`

**Interfaces:**
- Produces:
  - `cms_infra::storage::{Storage, LocalStorage, S3Storage, create_storage}`
  - `cms_infra::queue::{JobQueue, JobEnvelope, JobType, create_job_queue_with_pool, InMemoryJobQueue, PostgresJobQueue}`
  - `cms_infra::mailer::{Mailer, EmailRenderer, create_mailer, EmailMessage}`
  - `cms_infra::search::{SearchEngine, TantivySearchEngine, create_search_engine}`

- [x] **Step 1: Copy and adjust `storage` into `crates/cms-infra/src/storage/`**
- Copy `crates/cms-storage/src/*` into `crates/cms-infra/src/storage/`.
- Replace `cms_config::` with `crate::config::`.
- Replace `cms_error::` with `cms_error::AppError`.

- [x] **Step 2: Copy and adjust `queue` into `crates/cms-infra/src/queue/`**
- Copy `crates/cms-queue/src/*` into `crates/cms-infra/src/queue/`.
- Replace `cms_config::` with `crate::config::`.
- Ensure `reaper.rs` and types compile cleanly.

- [x] **Step 3: Copy and adjust `mailer` into `crates/cms-infra/src/mailer/`**
- Copy `crates/cms-mailer/src/*` into `crates/cms-infra/src/mailer/`.
- Replace `cms_config::` with `crate::config::` and `cms_i18n::` with `crate::i18n::`.
- Include `templates/` directory under `crates/cms-infra/src/mailer/templates/`.

- [x] **Step 4: Copy and adjust `search` into `crates/cms-infra/src/search/`**
- Copy `crates/cms-search/src/*` into `crates/cms-infra/src/search/`.
- Replace `cms_config::` with `crate::config::`.

- [x] **Step 5: Export all submodules in `crates/cms-infra/src/lib.rs`**
```rust
#![forbid(unsafe_code)]

pub mod analytics;
pub mod config;
pub mod i18n;
pub mod mailer;
pub mod queue;
pub mod search;
pub mod storage;

pub use analytics::{AnalyticsService, AnalyticsStore, PostgresAnalyticsStore};
pub use config::Config;
pub use i18n::{Locale, Catalog};
pub use mailer::{EmailRenderer, Mailer};
pub use queue::{JobQueue, JobEnvelope, JobType};
pub use search::SearchEngine;
pub use storage::Storage;
```

- [x] **Step 6: Run tests and compilation on `cms-infra`**

Run: `cargo test -p cms-infra`
Expected: PASS

- [x] **Step 7: Commit Task 2**

Run:
```bash
git add crates/cms-infra
git commit -m "feat(infra): integrate storage, queue, mailer, and search into cms-infra"
```

---

### Task 3: Migrate `crates/cms-server-core` to Consume `cms-infra`

**Files:**
- Modify: `crates/cms-server-core/Cargo.toml`
- Modify: `crates/cms-server-core/src/app_state.rs`
- Modify: `crates/cms-server-core/src/middleware/locale.rs`
- Modify: `crates/cms-server-core/src/middleware/rate_limit.rs`

**Interfaces:**
- Consumes: `cms-infra` (`cms_infra::config`, `cms_infra::storage`, `cms_infra::queue`, `cms_infra::search`, `cms_infra::mailer`, `cms_infra::analytics`, `cms_infra::i18n`)
- Produces: `AppState`, `BizContext`, server middleware suite

- [x] **Step 1: Update `crates/cms-server-core/Cargo.toml`**

Remove dependencies:
- `cms-config`, `cms-storage`, `cms-queue`, `cms-search`, `cms-analytics`, `cms-mailer`, `cms-i18n`
Add dependency:
- `cms-infra = { path = "../cms-infra" }`

- [x] **Step 2: Update `crates/cms-server-core/src/app_state.rs`**

Replace:
```rust
use cms_config::Config;
use cms_queue::JobQueue;
use cms_storage::Storage;
```
With:
```rust
use cms_infra::config::Config;
use cms_infra::queue::JobQueue;
use cms_infra::storage::Storage;
```
And replace field types:
- `pub search_engine: Arc<dyn cms_infra::search::SearchEngine>`
- `pub mailer: Arc<dyn cms_infra::mailer::Mailer>`
- `pub email_renderer: cms_infra::mailer::EmailRenderer`
Update `AppState::from_config` to invoke `cms_infra::storage::create_storage`, `cms_infra::queue::create_job_queue_with_pool`, etc.

- [x] **Step 3: Update `crates/cms-server-core/src/middleware/locale.rs`**

Update `cms_i18n::*` imports to `cms_infra::i18n::*`.

- [x] **Step 4: Verify `cargo check -p cms-server-core`**

Run: `cargo check -p cms-server-core`
Expected: PASS

- [x] **Step 5: Commit Task 3**

Run:
```bash
git add crates/cms-server-core
git commit -m "refactor(server-core): consume consolidated cms-infra"
```

---

### Task 4: Migrate `cms-server-services`, `cms-worker`, and Vertical Slices to `cms-infra`

**Files:**
- Modify: `crates/cms-server-services/Cargo.toml` and `src/storage_url.rs`, `src/event_dispatcher.rs`
- Modify: `crates/cms-worker/Cargo.toml` and `src/lib.rs`, `src/main.rs`
- Modify: `crates/cms-reports/Cargo.toml` and `src/lib.rs`
- Modify: `crates/cms-assets/Cargo.toml`
- Modify: `crates/cms-deployments/Cargo.toml`
- Modify: `crates/cms-git/Cargo.toml`

**Interfaces:**
- Consumes: `cms_infra::storage`, `cms_infra::queue`, `cms_infra::search`, `cms_infra::mailer`

- [x] **Step 1: Update `crates/cms-server-services`**
- In `Cargo.toml`: replace `cms-storage`, `cms-queue` with `cms-infra = { path = "../cms-infra" }`.
- In `src/storage_url.rs`: use `cms_infra::storage::Storage`.
- In `src/event_dispatcher.rs`: use `cms_infra::queue::JobQueue`.

- [x] **Step 2: Update `crates/cms-worker`**
- In `Cargo.toml`: replace `cms-storage`, `cms-queue`, `cms-search`, `cms-config`, `cms-analytics`, `cms-mailer` with `cms-infra = { path = "../cms-infra" }`.
- In worker code: update imports to `cms_infra::*`.

- [x] **Step 3: Update Vertical Slices (`cms-reports`, `cms-assets`, `cms-deployments`, `cms-git`)**
- Replace dependencies on individual infra crates with `cms-infra = { path = "../cms-infra" }`.
- Update import paths in source files.

- [x] **Step 4: Verify compilation**

Run: `cargo check -p cms-server-services -p cms-worker -p cms-reports -p cms-assets`
Expected: PASS

- [x] **Step 5: Commit Task 4**

Run:
```bash
git add crates/cms-server-services crates/cms-worker crates/cms-reports crates/cms-assets crates/cms-deployments crates/cms-git
git commit -m "refactor: update server services, worker, and vertical slices to cms-infra"
```

---

### Task 5: Eliminate `cms-middleware` Facade & Migrate Consumers to `cms-server-core`

**Files:**
- Modify: `crates/cms-worker/Cargo.toml` (remove `cms-middleware`, add `cms-server-core`)
- Modify: `crates/cms-sites/Cargo.toml` (remove `cms-middleware`, add `cms-server-core`)
- Modify: `crates/cms-notifications/Cargo.toml` (remove `cms-middleware`, add `cms-server-core`)
- Modify: `apps/api/Cargo.toml` (remove `cms-middleware`)
- Delete: `crates/cms-middleware/`

**Interfaces:**
- Consumes: `cms-server-core`
- Eliminates: `crates/cms-middleware`

- [x] **Step 1: Update consumers to import `cms-server-core` directly**
- In `crates/cms-worker/src/`: replace `cms_middleware::*` with `cms_server_core::*`.
- In `crates/cms-sites/src/`: replace `cms_middleware::*` with `cms_server_core::*`.
- In `crates/cms-notifications/src/`: replace `cms_middleware::*` with `cms_server_core::*`.

- [x] **Step 2: Remove `crates/cms-middleware` directory**

Delete directory `crates/cms-middleware`.

- [x] **Step 3: Verify compilation without `cms-middleware`**

Run: `cargo check -p cms-worker -p cms-sites -p cms-notifications`
Expected: PASS

- [x] **Step 4: Commit Task 5**

Run:
```bash
git add -A crates/
git commit -m "refactor: eliminate cms-middleware facade and migrate all callers to cms-server-core"
```

---

### Task 6: Update Composition Root (`apps/api`) and Workspace `Cargo.toml`

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `apps/api/Cargo.toml`
- Modify: `apps/api/lib.rs` / `apps/api/router.rs` / `apps/api/main.rs`

**Interfaces:**
- Produces: `cms-server` library and binary

- [x] **Step 1: Update workspace root `Cargo.toml`**
- In `[workspace.dependencies]`:
  - Add `cms-infra = { path = "crates/cms-infra" }`
  - Remove `cms-config`, `cms-storage`, `cms-search`, `cms-queue`, `cms-mailer`, `cms-analytics`, `cms-i18n`, `cms-middleware`
- Change downstream crates path dependencies to `{ workspace = true }` where standard.

- [x] **Step 2: Update `apps/api/Cargo.toml`**
- Replace individual infra dependencies and `cms-middleware` with `cms-infra = { workspace = true }` and `cms-server-core = { workspace = true }`.

- [x] **Step 3: Update `apps/api/lib.rs` and `apps/api/router.rs`**
- Update any remaining imports from `cms_config::` or other infra crates to `cms_infra::*`.

- [x] **Step 4: Verify `cms-server` compilation**

Run: `cargo check -p cms-server`
Expected: PASS

- [x] **Step 5: Commit Task 6**

Run:
```bash
git add Cargo.toml apps/api
git commit -m "chore(workspace): register cms-infra and update composition root"
```

---

### Task 7: Remove Old Crate Folders, Stage Legacy Git Deletions & Workspace Verification

**Files:**
- Delete: `crates/cms-config`
- Delete: `crates/cms-storage`
- Delete: `crates/cms-search`
- Delete: `crates/cms-queue`
- Delete: `crates/cms-mailer`
- Delete: `crates/cms-analytics`
- Delete: `crates/cms-i18n`
- Delete: `crates/cms-middleware`
- Modify: `AGENTS.md`, `README.md`

- [x] **Step 1: Remove the 7 obsolete crate directories**
Delete `crates/cms-config`, `crates/cms-storage`, `crates/cms-search`, `crates/cms-queue`, `crates/cms-mailer`, `crates/cms-analytics`, `crates/cms-i18n`.

- [x] **Step 2: Clean up git status for legacy `cms-biz` and `cms-api`**
Run `git add -u` to stage all deleted legacy files.

- [x] **Step 3: Update documentation in `AGENTS.md` and `README.md`**
Update crate diagrams and references in `AGENTS.md` to document `cms-infra` and remove `cms-middleware`.

- [x] **Step 4: Run full workspace verification**
Run:
```bash
cargo check --workspace
cargo test -p cms-infra
cargo test -p cms-server --lib
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: All commands exit with code 0.

- [x] **Step 5: Final commit**

Run:
```bash
git add -A
git commit -m "chore: purge obsolete infra micro-crates and finalize crate cleanup"
```
