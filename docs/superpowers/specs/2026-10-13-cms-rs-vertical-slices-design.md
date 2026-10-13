# Technical Design Spec: CMS Modular Monolith Vertical Slices Realization

- **Date**: 2026-10-13
- **Author**: Antigravity & Architecture Team
- **Status**: Approved
- **Target Repository**: `cms-rs-4`
- **Related ADR**: [ADR 0007: Modular Monolith Transition to Vertical Slices & Elimination of Horizontal Layers](../../adr/0007-vertical-slices-modular-monolith.md)

---

## 1. Context & Motivation

In [ADR 0007](../../adr/0007-vertical-slices-modular-monolith.md), `cms-rs-4` transitioned away from horizontal layers towards autonomous vertical slices. While domain areas such as `cms-content`, `cms-projects`, `cms-deployments`, `cms-git`, and `cms-invitations` successfully adopted this pattern, an architectural audit reveals that **Analytics** and **Search** were left incomplete:

1. **Orphaned Slices**:
   - `analytics` queries and services were placed in `crates/cms-infra/src/analytics/`, while its HTTP route handlers were dumped directly into `apps/api/router.rs`.
   - `search` indexing, tokenization, embeddings, and services were placed in `crates/cms-infra/src/search/`, while its HTTP route handlers were dumped directly into `apps/api/router.rs`.
2. **Infra Junk Drawer**:
   - `crates/cms-infra` has become overloaded with heavy ML, Japanese tokenization, Tantivy BM25, and markdown AST parsing dependencies alongside lightweight infrastructure (config, storage, mailer, queue).
3. **Composition Root Violation**:
   - `apps/api/router.rs` has grown to 510 lines, containing domain route handlers, inline subrouters, and domain TLS checks instead of remaining a purely declarative mounting root.
4. **Compilation Blast Radius**:
   - Because `cms-server-core::AppState` directly couples to `SearchEngine`, changes to search trigger transitive recompilation across all 28 crates in the workspace.

---

## 2. Goals & Non-Goals

### Goals
- **Full Autonomous Slices**: Extract `cms-analytics` and `cms-search` into dedicated vertical slice crates containing their own domain models, queries/services, and Axum routers (`pub fn router(state: Arc<AppState>) -> Router`).
- **Decoupled Search Engine**: Abstract `SearchEngine` behind a lightweight trait in `cms-entity` (or `cms-server-core`), removing heavy dependencies (`tantivy`, `lindera`, `fastembed`, `pulldown-cmark`) from `cms-infra`.
- **Pure Composition Root**: Clean `apps/api/router.rs` of all inline route handlers so it serves strictly as an assembly route table (~90 lines).
- **Workspace Modernization**: Upgrade `cms-infra/Cargo.toml` to `edition = "2024"`.
- **Zero Breakage**: Maintain 100% API compatibility with the React Studio frontend and existing reader sites.

### Non-Goals
- Modifying frontend endpoints or contract interfaces.
- Changing search engine backends or ranking algorithms.
- Re-introducing any billing, pricing, or marketing features (strictly forbidden by ADR 0001).

---

## 3. Architecture & Crate Structure

```
cms-rs-4/
├── apps/
│   └── api/                      # Pure Composition Root (ZERO domain route handlers)
│       ├── main.rs               # Binary entry point & graceful shutdown
│       └── router.rs             # Declarative router mounting all vertical slices
├── crates/
│   ├── cms-analytics/            # [NEW] Vertical Slice: Telemetry & Dashboards
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Exports router
│   │       ├── handlers.rs       # HTTP endpoints (track, events, query, dashboard, views)
│   │       ├── queries.rs        # PostgreSQL queries
│   │       └── services.rs       # AnalyticsService business logic
│   │
│   ├── cms-search/               # [NEW] Vertical Slice: Tantivy + Lindera + FastEmbed
│   │   ├── Cargo.toml            # Holds tantivy, lindera, fastembed, pulldown-cmark
│   │   └── src/
│   │       ├── lib.rs            # Exports router & SearchEngine implementation
│   │       ├── engine/
│   │       │   ├── tantivy.rs    # Tantivy BM25 inverted index
│   │       │   ├── tokenizer.rs  # Lindera Japanese tokenizer
│   │       │   ├── vector.rs     # FastEmbed embeddings & vector index
│   │       │   └── markdown.rs   # Markdown AST chunker
│   │       ├── rag.rs            # RAG context retriever
│   │       ├── services.rs       # SearchService
│   │       └── handlers.rs       # HTTP endpoints (search, reindex, runs, status)
│   │
│   ├── cms-infra/                # [LEAN] Pure infrastructure
│   │   ├── config/               # App configuration
│   │   ├── storage/              # Local & S3 StorageService
│   │   ├── queue/                # JobQueue (in-memory & Redis)
│   │   ├── mailer/               # Lettre & MiniJinja emailer
│   │   └── i18n/                 # Locale & direction detection
│   │
│   ├── cms-entity/               # Pure domain types, DTOs & abstract SearchEngine trait
│   ├── cms-server-core/          # AppState, enterprise middleware suite, pagination, error
│   └── [other vertical slices]   # cms-content, cms-projects, cms-deployments, etc.
```

---

## 4. Detailed Component Specifications

### 4.1 Crate: `cms-analytics`
- **Path**: `crates/cms-analytics`
- **Dependencies**:
  - `cms-entity`, `cms-db`, `cms-error`, `cms-server-core`
  - `axum`, `serde`, `serde_json`, `sqlx`, `chrono`, `uuid`, `tracing`
- **Exposed API**:
  - `pub fn router(state: Arc<AppState>) -> Router`
  - Mounted routes:
    - `POST /track` -> `track_event_handler`
    - `GET /events` -> `list_analytics_events_handler`
    - `POST /query` -> `query_analytics_handler`
    - `GET /dashboard/{project_id}` -> `get_analytics_dashboard_handler`
    - `GET /pages/{page_id}/views` -> `get_page_views_handler`

### 4.2 Crate: `cms-search`
- **Path**: `crates/cms-search`
- **Dependencies**:
  - `cms-entity`, `cms-db`, `cms-error`, `cms-server-core`, `cms-infra`
  - `tantivy`, `lindera`, `unicode-normalization`, `pulldown-cmark`, `fastembed`, `rusqlite`, `parking_lot`, `reqwest`, `axum`, `serde`, `serde_json`, `sqlx`, `chrono`, `uuid`, `tracing`
- **Exposed API**:
  - `pub fn router(state: Arc<AppState>) -> Router`
  - `pub struct TantivySearchEngine` (implements `cms_entity::search::SearchEngine`)
  - Mounted routes:
    - `GET /` -> `search_handler`
    - `POST /reindex` -> `reindex_handler`
    - `GET /index-runs` -> `list_search_index_runs_handler`
    - `GET /index-runs/{id}` -> `get_search_index_run_handler`
    - `GET /status/{project_id}` -> `get_search_status_handler`

### 4.3 `cms-infra` Slimming & Modernization
- **Path**: `crates/cms-infra`
- **Modifications**:
  - Delete `src/analytics/` directory.
  - Delete `src/search/` directory.
  - Delete dependencies: `tantivy`, `lindera`, `fastembed`, `pulldown-cmark`, `unicode-normalization`.
  - Update `Cargo.toml`: `edition = "2024"`.
  - Maintain `config`, `storage`, `queue`, `mailer`, and `i18n`.

### 4.4 Composition Root: `apps/api/router.rs`
- **Modifications**:
  - Remove all inline search & analytics handlers.
  - Mount `router.nest("/analytics", cms_analytics::router(state.clone()))`.
  - Mount `router.nest("/search", cms_search::router(state.clone()))`.
  - Move `check_domain_tls_handler` to `cms-projects::domains`.
  - Move `get_health` to `cms-server-core::health` or `cms-admin::platform_events`.

---

## 5. Verification Plan

1. **Compilation Check**:
   ```bash
   cargo check --workspace
   ```
2. **Strict Lint & Formatting**:
   ```bash
   cargo clippy --all-targets --all-features
   cargo fmt --check
   ```
3. **Integration Test Suite**:
   ```bash
   cargo test -p cms-analytics -p cms-search -p cms-server
   ```
4. **Frontend Verification**:
   ```bash
   bun run check
   bun run typecheck
   ```
