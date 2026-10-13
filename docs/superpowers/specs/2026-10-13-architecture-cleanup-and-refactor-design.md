# Technical Design Spec: Architecture Cleanup & Module Realization

- **Date**: 2026-10-13
- **Author**: Antigravity & Architecture Team
- **Status**: Approved
- **Target Repository**: `cms-rs-4`
- **Related ADR**: [ADR 0007: Modular Monolith Transition to Vertical Slices](../../adr/0007-vertical-slices-modular-monolith.md)

---

## 1. Problem Statement & Motivation

During a post-migration audit of the Modular Monolith transition, several redundant layers, duplicate routing structures, and misplaced methods were identified that increase maintenance burden:

1. **Composition Root Routing Duplication**:
   - `apps/api/router.rs` mounts 26 vertical slice subrouters at root (`/auth`, `/projects`, `/pages`, etc.), and then duplicates the exact same 26 `.nest(...)` route declarations verbatim into `app_router` mounted at `/app` and `/v1`.
2. **`cms-analytics` & `cms-worker` Dead Store Shims**:
   - `crates/cms-analytics/src/traits.rs` maintains an obsolete `AnalyticsStore` trait, while `services.rs` maintains `PostgresAnalyticsStore` and `create_analytics_store`.
   - `crates/cms-worker/src/lib.rs` initializes `WorkerState.analytics: Arc<dyn AnalyticsStore>` on startup, but never consumes it (jobs are processed via `cms_analytics::process_analytics_job`).
   - None of the other 15 vertical slices use store trait wrappers; they expose direct service functions accepting `&PgPool`.
3. **Misplaced Cross-Domain Method (`get_system_stats`)**:
   - `cms_analytics::AnalyticsService::get_system_stats` executes `SELECT COUNT(*) FROM "User"` and `SELECT COUNT(*) FROM "Project"`.
   - This platform-wide user/project administration metric was placed in `cms-analytics` and `cms-db::analytics`, causing `cms-admin` and `cms-workspaces` to couple directly to `cms-analytics`.
4. **`cms-search` Internal Layout Inconsistency**:
   - `rag.rs` sits at top-level `crates/cms-search/src/rag.rs`, while all other search engine internals (`tantivy.rs`, `vector.rs`, `tokenizer.rs`, `markdown.rs`) reside inside `src/engine/`.
   - `traits.rs` is a redundant 4-line re-export of `cms_server_core::search::SearchEngine`.
5. **Outdated Architecture Documentation**:
   - `AGENTS.md` still lists `search` and `analytics` under `cms-infra`, rather than as first-class vertical slices in the Modular Monolith catalog.

---

## 2. Design & Architectural Changes

### 2.1 Composition Root Deduplication (`apps/api/router.rs`)

Extract a private helper `build_domain_subrouter(state: &Arc<AppState>) -> Router` containing the single authoritative table of all 26 domain slices:

```rust
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
        .nest("/domains", cms_projects::domains_router().with_state(Arc::clone(state)))
        .nest("/reader-access", cms_projects::reader_access_router().with_state(Arc::clone(state)))
        .nest("/comments", cms_content::comment_router(Arc::clone(state)))
        .nest("/search", cms_search::router(Arc::clone(state)))
        .nest("/export", cms_reports::export_router(Arc::clone(state)))
        .nest("/openapi", cms_api_specs::router(Arc::clone(state)))
        .nest("/notifications", cms_notifications::router(Arc::clone(state)))
        .nest("/assets", cms_assets::router(Arc::clone(state)))
        .nest("/analytics", cms_analytics::router(Arc::clone(state)))
        .nest("/themes", cms_themes::router(Arc::clone(state)))
        .nest("/platform-events", cms_admin::platform_events::router(Arc::clone(state)))
        .nest("/mcp", cms_mcp::router(Arc::clone(state)))
        .nest("/members", cms_workspaces::members_router(Arc::clone(state)))
        .nest("/workspace", cms_workspaces::workspace_router(Arc::clone(state)))
        .nest("/invitations", cms_invitations::router(Arc::clone(state)))
        .nest("/reports", cms_reports::router(Arc::clone(state)))
        .nest("/setup", cms_onboarding::router(Arc::clone(state)))
}
```

In `create_api_router`:
```rust
let domain_router = build_domain_subrouter(&state);

let router = Router::new()
    .merge(domain_router.clone())
    .nest("/app", domain_router.clone())
    .nest("/v1", domain_router)
    .nest("/admin", cms_admin::router(Arc::clone(&state)))
    .nest("/public", cms_public::router(Arc::clone(&state)))
    .route("/api-docs/openapi.json", get(crate::docs::serve_openapi_spec))
    .route("/api-docs/openapi.yaml", get(crate::docs::serve_openapi_yaml))
    .route("/api-docs", get(crate::docs::serve_swagger_ui))
    .route("/api-docs/redoc", get(crate::docs::serve_redoc))
    .route("/health", get(cms_server_core::health::health_check_handler).with_state(Arc::clone(&state)))
    .route("/metrics", get(cms_admin::handlers::get_system_metrics_handler).with_state(Arc::clone(&state)))
    .nest("/v1/internal", cms_projects::tls_check_router(Arc::clone(&state)))
    .nest("/internal", cms_projects::tls_check_router(state));
```

### 2.2 Purify `cms-analytics` & `cms-worker`

1. **Delete Dead Store Layer**:
   - Delete `crates/cms-analytics/src/traits.rs`.
   - Delete `PostgresAnalyticsStore` and `create_analytics_store` from `crates/cms-analytics/src/services.rs`.
   - Export strictly from `crates/cms-analytics/src/lib.rs`:
     ```rust
     pub mod handlers;
     pub mod router;
     pub mod services;

     pub use router::router;
     pub use services::{process_analytics_job, AnalyticsService};
     ```
2. **Clean `WorkerState`**:
   - In `crates/cms-worker/src/lib.rs`, remove `pub analytics: Arc<dyn AnalyticsStore>` and all calls to `create_analytics_store`.
   - Retain `cms_analytics::process_analytics_job(&state.db, &job.payload).await`.
3. **Decouple `cms-workspaces`**:
   - Remove `cms-analytics` dependency from `crates/cms-workspaces/Cargo.toml` if no longer referenced.

### 2.3 Relocate Platform System Stats to `cms-admin`

1. **Move Database Query**:
   - In `crates/cms-db/src/platform_event.rs` (or `crates/cms-db/src/project.rs`), implement `get_system_stats(pool: &PgPool) -> Result<serde_json::Value, AppError>` querying `User` and `Project` counts.
   - Remove `get_system_stats` from `crates/cms-db/src/analytics.rs`.
2. **Move Business Service**:
   - In `crates/cms-admin/src/services.rs` (or create if missing, or in `cms-admin::platform_events`), provide `get_system_stats(pool: &PgPool) -> Result<serde_json::Value, AppError>`.
   - Update `crates/cms-admin/src/handlers.rs` to call `cms_admin`'s own service function.
   - Update `crates/cms-workspaces/src/handlers.rs` to call `cms_admin`'s service function (or `cms_db`).
   - Remove `get_system_stats` from `cms_analytics::AnalyticsService`.

### 2.4 Reorganize `cms-search` Internals

1. **Move RAG Module**:
   - Move `crates/cms-search/src/rag.rs` to `crates/cms-search/src/engine/rag.rs`.
   - Update `crates/cms-search/src/engine/mod.rs`:
     ```rust
     pub mod markdown;
     pub mod tantivy;
     pub mod tokenizer;
     #[cfg(feature = "vector")]
     pub mod vector;
     #[cfg(feature = "vector")]
     pub mod rag;
     ```
   - In `crates/cms-search/src/engine/tantivy.rs`: replace `crate::rag::generate_rag_answer` with `super::rag::generate_rag_answer`.
2. **Remove Redundant `traits.rs`**:
   - Delete `crates/cms-search/src/traits.rs`.
   - In `crates/cms-search/src/lib.rs`: export `pub use cms_server_core::search::SearchEngine;` directly.
   - Update re-exports in `engine/tantivy.rs` and `services.rs`.

### 2.5 Documentation & Architecture Parity (`AGENTS.md`)

- Update `AGENTS.md` platform overview to:
  - Remove `search` and `analytics` from `cms-infra` description.
  - Add explicit sections under "Vertical Slices & Platform Subsystems":
    - **Vertical Slice: `cms-analytics`**: Event ingestion, page views, query aggregations, project telemetry dashboards.
    - **Vertical Slice: `cms-search`**: Tantivy BM25 inverted index, Lindera Japanese morphology, FastEmbed vector embeddings, RAG answers.

---

## 3. Verification & Safety Constraints

1. **Compilation Check**:
   ```bash
   cargo check --workspace
   ```
2. **Clippy Compliance**:
   ```bash
   cargo clippy -p cms-analytics -p cms-search -p cms-worker -p cms-server -p cms-admin -p cms-workspaces
   ```
3. **Tests**:
   ```bash
   cargo test -p cms-analytics -p cms-search -p cms-server
   ```
4. **API Contract Invariance**:
   - Zero changes to external HTTP endpoint URLs, request models, or response DTOs.
