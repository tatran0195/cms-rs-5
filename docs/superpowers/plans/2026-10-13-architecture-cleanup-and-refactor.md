# Architecture Cleanup & Module Realization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clean up redundant routing structures, eliminate dead store trait shims in `cms-analytics` and `cms-worker`, relocate platform system stats from analytics to admin, consolidate `cms-search`'s engine module layout, and update repository documentation in `AGENTS.md`.

**Architecture:** Refactor `apps/api/router.rs` to build the domain subrouter once and mount it across `/`, `/app`, and `/v1`. Strip legacy `AnalyticsStore` / `PostgresAnalyticsStore` abstractions, standardizing `cms-analytics` on direct `AnalyticsService` like all other 15 vertical slices. Relocate `get_system_stats` into `cms-admin`, move `rag.rs` under `cms-search/src/engine/`, and ensure full workspace compilation and test passes.

**Tech Stack:** Rust 2024 / MSRV 1.85, Axum 0.8, SQLx, Tokio.

## Global Constraints

- Strict Clippy: `unsafe` forbidden, no `unwrap` or `panic` in production code.
- Zero changes to external API contracts, routes, and JSON schemas consumed by React Studio and reader sites.
- Clean compilation across all crates: `cargo check --workspace`.

---

### Task 1: Deduplicate Composition Root Router (`apps/api/router.rs`)

**Files:**
- Modify: `apps/api/router.rs:26-146`

**Interfaces:**
- Produces: `build_domain_subrouter(state: &Arc<AppState>) -> Router`
- Consumes: All 26 autonomous vertical slice routers (`cms_auth`, `cms_projects`, `cms_content`, etc.)

- [ ] **Step 1: Refactor `apps/api/router.rs` to extract `build_domain_subrouter`**

In `apps/api/router.rs`, define `build_domain_subrouter` and use it inside `create_api_router`:

```rust
/// Helper to construct the unified domain subrouter mounted at root, /app, and /v1
fn build_domain_subrouter(state: &Arc<AppState>) -> Router {
    Router::new()
        .nest("/auth", cms_auth::router(Arc::clone(state)))
        .nest("/projects", cms_projects::project_router(Arc::clone(state)))
        .nest("/pages", cms_content::page_router(Arc::clone(state)))
        .nest("/branches", cms_content::branch_router(Arc::clone(state)))
        .nest("/languages", cms_content::language_router(Arc::clone(state)))
        .nest("/git", cms_git::router(Arc::clone(state)))
        .nest("/integrations", cms_integrations::router(Arc::clone(state)))
        .nest("/deployments", cms_deployments::router(Arc::clone(state)))
        .nest(
            "/domains",
            cms_projects::domains_router().with_state(Arc::clone(state)),
        )
        .nest(
            "/reader-access",
            cms_projects::reader_access_router().with_state(Arc::clone(state)),
        )
        .nest("/comments", cms_content::comment_router(Arc::clone(state)))
        .nest("/search", cms_search::router(Arc::clone(state)))
        .nest("/export", cms_reports::export_router(Arc::clone(state)))
        .nest("/openapi", cms_api_specs::router(Arc::clone(state)))
        .nest("/notifications", cms_notifications::router(Arc::clone(state)))
        .nest("/assets", cms_assets::router(Arc::clone(state)))
        .nest("/analytics", cms_analytics::router(Arc::clone(state)))
        .nest("/themes", cms_themes::router(Arc::clone(state)))
        .nest(
            "/platform-events",
            cms_admin::platform_events::router(Arc::clone(state)),
        )
        .nest("/mcp", cms_mcp::router(Arc::clone(state)))
        .nest("/members", cms_workspaces::members_router(Arc::clone(state)))
        .nest(
            "/workspace",
            cms_workspaces::workspace_router(Arc::clone(state)),
        )
        .nest("/invitations", cms_invitations::router(Arc::clone(state)))
        .nest("/reports", cms_reports::router(Arc::clone(state)))
        .nest("/setup", cms_onboarding::router(Arc::clone(state)))
}

/// Create the un-middlewared root API router assembling all vertical slices
pub fn create_api_router(state: Arc<AppState>) -> Router {
    let domain_router = build_domain_subrouter(&state);

    let mut router = Router::new()
        .merge(domain_router.clone())
        .nest("/app", domain_router.clone())
        .nest("/v1", domain_router);

    // Admin routes (platform-admin-only)
    router = router.nest("/admin", cms_admin::router(Arc::clone(&state)));

    // Public routes (unauthenticated reader-facing JSON API)
    router = router.nest("/public", cms_public::router(Arc::clone(&state)));

    // OpenAPI documentation endpoints
    router = router.route(
        "/api-docs/openapi.json",
        get(crate::docs::serve_openapi_spec),
    );
    router = router.route(
        "/api-docs/openapi.yaml",
        get(crate::docs::serve_openapi_yaml),
    );
    router = router.route("/api-docs", get(crate::docs::serve_swagger_ui));
    router = router.route("/api-docs/redoc", get(crate::docs::serve_redoc));

    // Health check endpoint
    router = router.route(
        "/health",
        get(cms_server_core::health::health_check_handler).with_state(Arc::clone(&state)),
    );

    // Prometheus metrics scrape endpoint
    router = router.route(
        "/metrics",
        get(cms_admin::handlers::get_system_metrics_handler).with_state(Arc::clone(&state)),
    );

    // Caddy on-demand TLS check internal endpoints
    router = router.nest("/v1/internal", cms_projects::tls_check_router(Arc::clone(&state)));
    router = router.nest("/internal", cms_projects::tls_check_router(state));

    router
}
```

- [ ] **Step 2: Verify `cms-server` compilation**

Run: `cargo check -p cms-server`
Expected: PASS

- [ ] **Step 3: Commit router deduplication**

```bash
git add apps/api/router.rs
git commit -m "refactor(api): eliminate duplicate subrouter nesting in composition root"
```

---

### Task 2: Relocate Platform System Stats to `cms-admin`

**Files:**
- Modify: `crates/cms-db/src/platform_event.rs` (add `get_system_stats` query)
- Modify: `crates/cms-db/src/analytics.rs:300-318` (remove `get_system_stats`)
- Modify: `crates/cms-admin/src/handlers.rs` (update system stats caller)
- Modify: `crates/cms-workspaces/src/handlers.rs` (update system stats caller)
- Modify: `crates/cms-workspaces/Cargo.toml` (remove `cms-analytics` dependency)
- Modify: `crates/cms-analytics/src/services.rs:174-178` (remove `get_system_stats`)

**Interfaces:**
- Produces: `cms_db::platform_event::PlatformEventQueries::get_system_stats(pool: &PgPool) -> Result<serde_json::Value, AppError>`
- Consumes: PostgreSQL connection pool

- [ ] **Step 1: Add `get_system_stats` to `crates/cms-db/src/platform_event.rs`**

```rust
impl PlatformEventQueries {
    /// Get system-wide statistics (total users and total projects)
    pub async fn get_system_stats(pool: &PgPool) -> Result<serde_json::Value, AppError> {
        let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"User\"")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

        let projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"Project\"")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

        Ok(serde_json::json!({
            "users": users,
            "projects": projects,
        }))
    }
}
```

- [ ] **Step 2: Remove `get_system_stats` from `crates/cms-db/src/analytics.rs` and `crates/cms-analytics/src/services.rs`**

Delete `AnalyticsQueries::get_system_stats` from `crates/cms-db/src/analytics.rs:300-318`.
Delete `AnalyticsService::get_system_stats` from `crates/cms-analytics/src/services.rs:174-178`.

- [ ] **Step 3: Update `crates/cms-admin/src/handlers.rs` and `crates/cms-workspaces/src/handlers.rs`**

In `crates/cms-admin/src/handlers.rs`:
Replace `cms_analytics::AnalyticsService::get_system_stats(state.db())` with `cms_db::platform_event::PlatformEventQueries::get_system_stats(state.db())`.

In `crates/cms-workspaces/src/handlers.rs`:
Replace `cms_analytics::AnalyticsService::get_system_stats(state.db())` with `cms_db::platform_event::PlatformEventQueries::get_system_stats(state.db())`.

In `crates/cms-workspaces/Cargo.toml`:
Remove `cms-analytics = { workspace = true }`.

- [ ] **Step 4: Verify compilation across modified crates**

Run: `cargo check -p cms-db -p cms-analytics -p cms-admin -p cms-workspaces`
Expected: PASS

- [ ] **Step 5: Commit system stats relocation**

```bash
git add crates/cms-db crates/cms-analytics crates/cms-admin crates/cms-workspaces
git commit -m "refactor(admin): relocate get_system_stats from cms-analytics to cms-db platform_event"
```

---

### Task 3: Eliminate Obsolete `AnalyticsStore` & Purify `cms-analytics` and `cms-worker`

**Files:**
- Delete: `crates/cms-analytics/src/traits.rs`
- Modify: `crates/cms-analytics/src/services.rs:200-284` (remove `PostgresAnalyticsStore` and `create_analytics_store`)
- Modify: `crates/cms-analytics/src/lib.rs` (clean exports)
- Modify: `crates/cms-worker/src/lib.rs` (remove `analytics` field from `WorkerState`)

**Interfaces:**
- Produces: `cms_analytics::AnalyticsService`, `cms_analytics::process_analytics_job`, `cms_analytics::router`
- Removes: `cms_analytics::AnalyticsStore`, `cms_analytics::PostgresAnalyticsStore`, `cms_analytics::create_analytics_store`

- [ ] **Step 1: Delete `crates/cms-analytics/src/traits.rs`**

Remove the file `crates/cms-analytics/src/traits.rs`.

- [ ] **Step 2: Clean `crates/cms-analytics/src/services.rs` and `crates/cms-analytics/src/lib.rs`**

In `crates/cms-analytics/src/services.rs`:
Remove `use crate::traits::AnalyticsStore;`.
Delete `PostgresAnalyticsStore` and `create_analytics_store` (lines 203–284).

In `crates/cms-analytics/src/lib.rs`:
```rust
//! CMS Analytics Vertical Slice
//!
//! Autonomous domain crate handling event tracking, metrics aggregation,
//! page view counters, and project dashboards.

pub mod handlers;
pub mod router;
pub mod services;

pub use router::router;
pub use services::{process_analytics_job, AnalyticsService};
```

- [ ] **Step 3: Remove unused `analytics` field from `WorkerState` in `crates/cms-worker/src/lib.rs`**

In `crates/cms-worker/src/lib.rs`:
1. Remove `use cms_analytics::{AnalyticsStore, create_analytics_store};`.
2. In `WorkerState`: remove `pub analytics: Arc<dyn AnalyticsStore>,`.
3. In `WorkerState::with_pool`: remove `let analytics = create_analytics_store(...).await?;` and `analytics,`.
4. In `WorkerState::with_app_state`: remove `let analytics = create_analytics_store(...).await?;` and `analytics,`.

- [ ] **Step 4: Verify `cms-analytics` and `cms-worker` compilation**

Run: `cargo check -p cms-analytics -p cms-worker`
Expected: PASS

- [ ] **Step 5: Commit `AnalyticsStore` elimination**

```bash
git add crates/cms-analytics crates/cms-worker
git commit -m "refactor(analytics): remove obsolete AnalyticsStore traits and unused WorkerState field"
```

---

### Task 4: Consolidate `cms-search` Internal Layout (RAG under `engine/` & clean `traits.rs`)

**Files:**
- Move: `crates/cms-search/src/rag.rs` -> `crates/cms-search/src/engine/rag.rs`
- Delete: `crates/cms-search/src/traits.rs`
- Modify: `crates/cms-search/src/engine/mod.rs`
- Modify: `crates/cms-search/src/engine/tantivy.rs`
- Modify: `crates/cms-search/src/lib.rs`
- Modify: `crates/cms-search/src/services.rs`

**Interfaces:**
- Produces: `cms_search::engine::rag::generate_rag_answer`, `cms_search::SearchEngine`
- Consumes: `cms_server_core::search::SearchEngine`

- [ ] **Step 1: Move `rag.rs` to `crates/cms-search/src/engine/rag.rs`**

Move `crates/cms-search/src/rag.rs` to `crates/cms-search/src/engine/rag.rs`.
Delete `crates/cms-search/src/traits.rs`.

- [ ] **Step 2: Update `crates/cms-search/src/engine/mod.rs`**

```rust
//! Search engine components: Tantivy inverted index, Lindera tokenizer, vector embeddings, Markdown chunker, and RAG.

pub mod markdown;
#[cfg(feature = "vector")]
pub mod rag;
pub mod tantivy;
pub mod tokenizer;
#[cfg(feature = "vector")]
pub mod vector;
```

- [ ] **Step 3: Update `crates/cms-search/src/engine/tantivy.rs`**

In `crates/cms-search/src/engine/tantivy.rs:45`:
Replace `use crate::traits::SearchEngine;` with `use cms_server_core::search::SearchEngine;`.

In `crates/cms-search/src/engine/tantivy.rs:811`:
Replace `crate::rag::generate_rag_answer` with `super::rag::generate_rag_answer`.

- [ ] **Step 4: Update `crates/cms-search/src/lib.rs` and `crates/cms-search/src/services.rs`**

In `crates/cms-search/src/lib.rs`:
```rust
pub mod engine;
pub mod handlers;
pub mod router;
pub mod services;

pub use engine::{markdown, tantivy, tokenizer};
#[cfg(feature = "vector")]
pub use engine::rag;
#[cfg(feature = "vector")]
pub use engine::vector::{Embedder, ProjectVectorIndex, VectorMeta, VectorSearchResult};
pub use engine::tantivy::{ProjectIndex, TantivyFields, TantivySearchEngine};
pub use engine::tokenizer::{JapaneseToken, JapaneseTokenizer, LinderaTantivyTokenizer};
pub use router::router;
pub use services::{process_search_job, SearchService};
pub use cms_server_core::search::SearchEngine;
```

In `crates/cms-search/src/services.rs:18`:
Replace `use super::SearchEngine;` with `use cms_server_core::search::SearchEngine;`.

- [ ] **Step 5: Verify `cms-search` compilation and tests**

Run: `cargo test -p cms-search`
Expected: PASS

- [ ] **Step 6: Commit search engine layout consolidation**

```bash
git add crates/cms-search
git commit -m "refactor(search): relocate rag module under engine/ and streamline SearchEngine exports"
```

---

### Task 5: Update Platform Documentation & Full Workspace Verification

**Files:**
- Modify: `AGENTS.md` (correct `cms-infra` description and add `cms-analytics` & `cms-search` vertical slices)

- [ ] **Step 1: Update `AGENTS.md`**

In `AGENTS.md`:
1. Line 60: Change description of `cms-infra` to:
   `├── cms-infra/ # Consolidated infrastructure (config with validation, storage with StorageService, queue, mailer, i18n)`
2. Under Section "Vertical Slices & Platform Subsystems (Modular Monolith)", add:
   - **Vertical Slice: `cms-analytics`**: Autonomous domain crate encapsulating event tracking, page view counts, query aggregation, and project telemetry dashboards.
   - **Vertical Slice: `cms-search`**: Autonomous domain crate encapsulating isolated Tantivy BM25 inverted indexes, Lindera Japanese morphological analysis, FastEmbed vector embeddings, and RAG context generation.

- [ ] **Step 2: Run full workspace compilation check**

Run: `cargo check --workspace`
Expected: PASS with 0 errors.

- [ ] **Step 3: Run target test suites**

Run: `cargo test -p cms-analytics -p cms-search -p cms-server`
Expected: PASS

- [ ] **Step 4: Commit documentation and complete verification**

```bash
git add AGENTS.md
git commit -m "docs: align AGENTS.md with autonomous cms-analytics and cms-search vertical slices"
```
