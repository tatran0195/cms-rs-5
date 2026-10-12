# Modular Monolith: Domain Vertical Slices & Crate Decommissioning Design

- **Status**: Draft
- **Date**: 2026-10-12
- **Author**: Antigravity (Agentic AI) & Architecture Team
- **Target Branch**: `feat/vertical-slices-subsystems`

---

## 1. Executive Summary & Goals

Following the successful extraction of platform subsystem vertical slices (`cms-notifications`, `cms-invitations`, `cms-onboarding`, `cms-reports`) and the shared services foundation (`cms-server-services`), the platform architecture is evolving from the legacy 3-layer split (`cms-biz` / `cms-api`) into a cohesive **Modular Monolith** vertical slice architecture.

### Primary Objectives:
1. **Eliminate Artificial Crate Layering**: The horizontal split between `cms-biz` and `cms-api` creates boilerplate pass-through calls, rigid boundary ceremonies, and split domain logic.
2. **High Cohesion per Domain**: Colocate domain models, database queries, business services, and Axum route handlers within self-contained domain vertical slices.
3. **Decommission `cms-biz` and `cms-api`**: Systematically extract remaining domains into vertical slices and safely delete the `crates/cms-biz` and `crates/cms-api` crates.
4. **Preserve Compatibility & Zero Downtime**: Ensure every existing route, URL path, and HTTP contract remains 100% backward-compatible throughout the migration.

---

## 2. Phased Roadmap

To manage complexity and verify every milestone with strict compiler and Clippy gates, the migration is organized into 4 sequential phases:

```mermaid
graph TD
    subgraph Phase 1: Content Core
        C1[crates/cms-content]
        M1[pages, branches, languages, comments]
    end

    subgraph Phase 2: Workspaces & Assets
        C2[crates/cms-projects]
        C3[crates/cms-assets]
    end

    subgraph Phase 3: Publishing & Git Sync
        C4[crates/cms-deployments]
        C5[crates/cms-git-sync]
    end

    subgraph Phase 4: Dissolution & Assembly
        C6[Direct Router Folding: cms-search, cms-mcp, cms-analytics, cms-auth]
        C7[Delete cms-biz & cms-api]
        C8[Direct Assembly in cms-server]
    end

    Phase 1 --> Phase 2
    Phase 2 --> Phase 3
    Phase 3 --> Phase 4
```

| Phase | Target Crate(s) | Extracted Domains | Status |
|---|---|---|---|
| **Phase 1** | `crates/cms-content` | Pages, Revisions, Branches, Languages, Comments | **Current Focus** |
| **Phase 2** | `crates/cms-projects`, `crates/cms-assets` | Projects, Workspaces, Reader Access, Asset Storage | Pending |
| **Phase 3** | `crates/cms-deployments`, `crates/cms-git-sync` | Deployments, Custom Domains, Git Sync, Integrations | Pending |
| **Phase 4** | Infrastructure Crates & `cms-server` | Search, MCP, Analytics, Auth, Admin routers folded; `cms-biz` and `cms-api` removed | Pending |

---

## 3. Phase 1 Specification: `crates/cms-content`

`crates/cms-content` consolidates the core documentation authoring and version control engine into a single vertical slice crate.

### 3.1 Domain Boundary & Submodules

```
crates/cms-content/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public exports & route mounting
│   ├── models/             # Domain DTOs and database models
│   │   ├── mod.rs
│   │   ├── page.rs
│   │   ├── branch.rs
│   │   ├── language.rs
│   │   └── comment.rs
│   ├── queries/            # Database queries (SQLx)
│   │   ├── mod.rs
│   │   ├── page.rs
│   │   ├── branch.rs
│   │   ├── language.rs
│   │   └── comment.rs
│   ├── services/           # Business logic & domain coordination
│   │   ├── mod.rs
│   │   ├── page.rs         # PageService (tree resolution, revisions, slug indexing)
│   │   ├── branch.rs       # BranchService (branching, merging, default branch protection)
│   │   ├── language.rs     # LanguageService (multilingual fallbacks, locale binding)
│   │   └── comment.rs      # CommentService (inline annotations, threaded discussions)
│   └── handlers/           # Axum HTTP routes and extractors
│       ├── mod.rs
│       ├── page.rs         # /pages endpoints
│       ├── branch.rs       # /branches endpoints
│       ├── language.rs     # /languages endpoints
│       └── comment.rs      # /comments endpoints
└── tests/
    ├── page_test.rs
    ├── branch_test.rs
    ├── language_test.rs
    └── comment_test.rs
```

### 3.2 Public API & Subrouters

`cms-content` exposes both individual subrouters and a composite content router:

```rust
pub fn page_router(state: Arc<AppState>) -> Router;
pub fn branch_router(state: Arc<AppState>) -> Router;
pub fn language_router(state: Arc<AppState>) -> Router;
pub fn comment_router(state: Arc<AppState>) -> Router;

/// Composite content router mounting all 4 subrouters
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/pages", page_router(state.clone()))
        .nest("/branches", branch_router(state.clone()))
        .nest("/languages", language_router(state.clone()))
        .nest("/comments", comment_router(state.clone()))
}
```

### 3.3 HTTP Route Endpoints

#### Pages (`/pages`)
- `GET /` — List pages (filtered by project, branch, language)
- `POST /` — Create page
- `GET /{id}` — Get page with content / revision metadata
- `PUT /{id}` — Update page metadata or markdown body
- `DELETE /{id}` — Delete page

#### Branches (`/branches`)
- `GET /` — List branches for project
- `POST /` — Create branch
- `GET /{id}` — Get branch details
- `PUT /{id}` — Update branch (name, slug)
- `DELETE /{id}` — Delete branch (protected default branch rule)

#### Languages (`/languages`)
- `GET /` — List supported languages for project
- `POST /` — Add language to project
- `GET /{id}` — Get language details
- `PUT /{id}` — Update language
- `DELETE /{id}` — Remove language
- `POST /projects/{project_id}/set-default` — Set default project language

#### Comments (`/comments`)
- `GET /` — List comments
- `POST /` — Create comment (page annotation or reply)
- `GET /{id}` — Get comment
- `PUT /{id}` — Update comment
- `DELETE /{id}` — Delete comment
- `POST /{id}/resolve` — Toggle resolved state
- `GET /pages/{page_id}` — List threaded comments for page

---

## 4. Dependencies & Migration Plan

### 4.1 Crate Dependencies (`crates/cms-content/Cargo.toml`)
- `cms-db`: Database connection pool & existing SQLx primitives
- `cms-entity`: Domain models (`Page`, `Branch`, `Language`, `Comment`)
- `cms-error`: `AppError`
- `cms-server-core`: `AppState`, pagination, request tracing
- `cms-server-services`: `TenantContextService`
- `axum`, `tokio`, `serde`, `serde_json`, `uuid`, `chrono`, `sqlx`, `tracing`

### 4.2 Incremental Integration
1. Scaffold `crates/cms-content` with all models, queries, services, handlers, and unit tests.
2. Update `crates/cms-api/src/page/mod.rs` to delegate to `cms_content::page_router(state)`.
3. Update `crates/cms-api/src/branch/mod.rs` to delegate to `cms_content::branch_router(state)`.
4. Update `crates/cms-api/src/language/mod.rs` to delegate to `cms_content::language_router(state)`.
5. Update `crates/cms-api/src/comment/mod.rs` to delegate to `cms_content::comment_router(state)`.
6. Remove redundant domain files (`page.rs`, `branch.rs`, `language.rs`, `comment.rs`) from `cms-biz` and obsolete handler bodies from `cms-api`.
7. Verify workspace check and test suites pass (`cargo test -p cms-content -j 2`, `cargo check --workspace -j 2`).

---

## 5. Non-Negotiable Constraints & Quality Gates

1. **Rust Edition 2024 / MSRV 1.85**: `forbid(unsafe_code)`.
2. **Clippy Compliance**: 0 warnings with `clippy::all` denied.
3. **No Placeholders or Stubs**: Production-ready implementation with proper error propagation (`?`).
4. **ADR 0001 Scope**: No billing, plan tiers, payment gateways, or public marketing features.
5. **Windows Parallelism Constraint**: All builds run with `-j 2` to prevent memory/buffer panics.
