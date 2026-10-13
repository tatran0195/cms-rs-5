# CMS Modular Monolith Vertical Slices Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fully realize the Modular Monolith & Vertical Slices architecture in `cms-rs-4` by extracting `cms-analytics` and `cms-search` into autonomous domain crates, decoupling search dependencies from `cms-infra`, and restoring `apps/api` as a pure declarative Composition Root.

**Architecture:** Decompose `cms-infra` into a lean infrastructure crate while promoting Analytics and Search to high-cohesion vertical slices with encapsulated services, database queries, and Axum subrouters. Abstract search engine interfaces via `cms-entity` to break compilation blast radius across the 28 workspace member crates.

**Tech Stack:** Rust (Edition 2024, MSRV 1.85), Axum 0.8, SQLx, Tantivy 0.26, Lindera 6.0, FastEmbed 7.1, Tokio 1.x, Tower 0.5.

## Global Constraints
- Dedicated internal company documentation platform (NO Plan, Billing, Pricing, or Marketing features per ADR 0001).
- Edition 2024, Rust 1.85 MSRV across all crates.
- Strict clippy: `forbid(unsafe_code)`, `clippy::all` denied, no `unwrap` or `panic`.
- Every domain crate must expose `pub fn router(state: Arc<AppState>) -> axum::Router`.
- `apps/api` must contain ZERO domain HTTP handlers; it must only assemble subrouters.

---

### Task 1: Scaffold `crates/cms-analytics` Domain Crate

**Files:**
- Create: `crates/cms-analytics/Cargo.toml`
- Create: `crates/cms-analytics/src/lib.rs`
- Create: `crates/cms-analytics/src/queries.rs`
- Create: `crates/cms-analytics/src/services.rs`
- Modify: `Cargo.toml:18-48` (register `cms-analytics` in workspace dependencies)

**Interfaces:**
- Consumes: `cms_entity::analytics::*`, `cms_db::PgPool`, `cms_error::AppError`
- Produces: `cms_analytics::services::AnalyticsService`

- [ ] **Step 1: Register workspace member in root Cargo.toml**

Add `cms-analytics` to `[workspace.dependencies]` in root `Cargo.toml`:
```toml
cms-analytics = { path = "crates/cms-analytics" }
```

- [ ] **Step 2: Create `crates/cms-analytics/Cargo.toml`**

```toml
[package]
name = "cms-analytics"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Vertical Slice: Analytics telemetry, queries, dashboards, and page view metrics"

[dependencies]
cms-entity = { workspace = true }
cms-db = { workspace = true }
cms-error = { workspace = true }
cms-server-core = { workspace = true }

axum = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
sqlx = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["full"] }

[lints]
workspace = true
```

- [ ] **Step 3: Migrate queries and services into `cms-analytics`**

Copy `crates/cms-infra/src/analytics/postgres.rs` to `crates/cms-analytics/src/queries.rs`.
Copy `crates/cms-infra/src/analytics/services.rs` to `crates/cms-analytics/src/services.rs`.
Create `crates/cms-analytics/src/lib.rs`:
```rust
//! CMS Analytics Vertical Slice
//!
//! Handles event tracking, metrics aggregation, page view counters, and project dashboards.

pub mod queries;
pub mod services;
pub mod handlers;
pub mod router;

pub use router::router;
pub use services::AnalyticsService;
```

- [ ] **Step 4: Verify `cms-analytics` compiles**

Run: `cargo check -p cms-analytics`
Expected: PASS (0 errors).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/cms-analytics
git commit -m "feat(analytics): scaffold cms-analytics vertical slice crate"
```

---

### Task 2: Implement Handlers & Router in `cms-analytics`

**Files:**
- Create: `crates/cms-analytics/src/handlers.rs`
- Create: `crates/cms-analytics/src/router.rs`
- Modify: `crates/cms-analytics/src/lib.rs`

**Interfaces:**
- Consumes: `cms_server_core::app_state::AppState`, `cms_auth::AuthExtractor`
- Produces: `cms_analytics::router(state: Arc<AppState>) -> axum::Router`

- [ ] **Step 1: Implement `crates/cms-analytics/src/handlers.rs`**

Move handlers from `apps/api/router.rs:168-226` to `crates/cms-analytics/src/handlers.rs`:
- `track_event_handler`
- `list_analytics_events_handler`
- `query_analytics_handler`
- `get_analytics_dashboard_handler`
- `get_page_views_handler`

- [ ] **Step 2: Implement `crates/cms-analytics/src/router.rs`**

```rust
use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use cms_server_core::app_state::AppState;
use crate::handlers::*;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/track", post(track_event_handler))
        .route("/events", get(list_analytics_events_handler))
        .route("/query", post(query_analytics_handler))
        .route("/dashboard/{project_id}", get(get_analytics_dashboard_handler))
        .route("/pages/{page_id}/views", get(get_page_views_handler))
        .with_state(state)
}
```

- [ ] **Step 3: Run cargo check to verify handlers compile**

Run: `cargo check -p cms-analytics`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/cms-analytics
git commit -m "feat(analytics): add HTTP handlers and subrouter to cms-analytics"
```

---

### Task 3: Mount `cms-analytics` in `apps/api` and Clean Up `cms-infra`

**Files:**
- Modify: `apps/api/Cargo.toml` (add `cms-analytics = { workspace = true }`)
- Modify: `apps/api/router.rs` (remove analytics handlers, mount `cms_analytics::router`)
- Modify: `crates/cms-infra/src/lib.rs` (remove `pub mod analytics`)
- Delete: `crates/cms-infra/src/analytics/`

**Interfaces:**
- Consumes: `cms_analytics::router`
- Produces: `apps/api::create_api_router` mounting `/analytics` via `cms_analytics::router`

- [ ] **Step 1: Add dependency to `apps/api/Cargo.toml`**

Add `cms-analytics = { workspace = true }` under `[dependencies]` in `apps/api/Cargo.toml`.

- [ ] **Step 2: Replace inline handlers in `apps/api/router.rs`**

In `apps/api/router.rs`:
- Delete `track_event_handler`, `list_analytics_events_handler`, `query_analytics_handler`, `get_analytics_dashboard_handler`, `get_page_views_handler`, and `analytics_router`.
- Replace `.nest("/analytics", analytics_router(state.clone()))` with `.nest("/analytics", cms_analytics::router(state.clone()))`.

- [ ] **Step 3: Remove analytics module from `crates/cms-infra`**

Remove `pub mod analytics;` from `crates/cms-infra/src/lib.rs`.
Remove directory `crates/cms-infra/src/analytics/`.

- [ ] **Step 4: Verify workspace compilation**

Run: `cargo check --workspace`
Expected: PASS with 0 errors.

- [ ] **Step 5: Commit**

```bash
git add apps/api crates/cms-infra crates/cms-analytics
git commit -m "refactor(analytics): mount cms-analytics in api and remove from cms-infra"
```

---

### Task 4: Abstract `SearchEngine` Trait in `cms-entity`

**Files:**
- Modify: `crates/cms-entity/src/search.rs`
- Modify: `crates/cms-server-core/src/app_state.rs`

**Interfaces:**
- Consumes: `cms_entity::search::*`
- Produces: `pub trait SearchEngine: Send + Sync` in `cms-entity::search`

- [x] **Step 1: Define `SearchEngine` trait in `crates/cms-server-core/src/search.rs`**
- [x] **Step 2: Update `AppState` to reference decoupled `SearchEngine`**
- [x] **Step 3: Verify `cms-server-core` compiles without `cms-infra` search coupling**
- [x] **Step 4: Commit**

---

### Task 5: Scaffold `crates/cms-search` Vertical Slice

- [x] **Step 1: Register workspace member in root Cargo.toml**
- [x] **Step 2: Create `crates/cms-search/Cargo.toml`**
- [x] **Step 3: Migrate search files from `cms-infra` into `cms-search`**
- [x] **Step 4: Verify `cms-search` compiles**
- [x] **Step 5: Commit**

---

### Task 6: Implement Handlers & Router in `cms-search`

- [x] **Step 1: Implement `crates/cms-search/src/handlers.rs`**
- [x] **Step 2: Implement `crates/cms-search/src/router.rs`**
- [x] **Step 3: Verify `cms-search` compilation**
- [x] **Step 4: Commit**

---

### Task 7: Mount `cms-search` in `apps/api` and Slim `cms-infra`

- [x] **Step 1: Add `cms-search` to `apps/api/Cargo.toml`**
- [x] **Step 2: Update `apps/api/router.rs`**
- [x] **Step 3: Remove search engine dependencies from `crates/cms-infra/Cargo.toml`**
- [x] **Step 4: Verify workspace compilation**
- [x] **Step 5: Commit**

---

### Task 8: Composition Root Final Cleanup & `cms-infra` Edition 2024

- [x] **Step 1: Move `check_domain_tls_handler` to `cms-projects`**
- [x] **Step 2: Move `get_health` to `cms-server-core::health`**
- [x] **Step 3: Simplify `apps/api/router.rs`**
- [x] **Step 4: Upgrade `cms-infra` edition to 2024**
- [x] **Step 5: Verify workspace compilation**
- [x] **Step 6: Commit**

---

### Task 9: Full Verification Suite

- [x] **Step 1: Run Cargo Check across workspace**
- [x] **Step 2: Run Cargo Clippy across workspace**
- [x] **Step 3: Run Backend Tests**
- [x] **Step 4: Run Frontend Lint & Typecheck**
- [x] **Step 5: Final Git Status Verification**

