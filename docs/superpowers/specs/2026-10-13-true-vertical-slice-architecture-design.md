# Design Document: True Vertical Slice Architecture Migration

- **Date:** 2026-10-13
- **Status:** Approved
- **Audience:** Core Platform & Backend Engineering

---

## 1. Problem Statement & Motivation

A critical architectural audit of `cms-rs-4` revealed that despite having domain crate folders, the system is fundamentally structured as a **horizontal layered monolith** disguised as vertical slices:

1. **Centralized Kitchen-Sink Data Layer (`cms-db`):** Contains SQL queries, schema representations, and table queries for all 15+ domains (`page.rs`, `project.rs`, `deployment.rs`, `git.rs`, `integration.rs`, etc.).
2. **Centralized Entity Junk Drawer (`cms-entity`):** Contains domain models, DTOs, and serialization representations for all entities across the entire system.
3. **Empty Re-export Shims:** Domain crates contain placeholder files like `cms-content/src/queries/page.rs` (`pub use cms_db::page::PageQueries;`) and `cms-content/src/models/page.rs` (`pub use cms_entity::page::*;`), giving a false impression of modularity while preserving 100% coupling to horizontal repositories.
4. **Duplicate Implementations:** `cms-projects` duplicated ~700 lines of SQL queries from `cms-db/src/project.rs`. 7 domain crates duplicated `FromRequestParts` authentication extractors (`DeploymentUser`, `ContentUser`, `ProjectUser`, `GitUser`, `IntegrationUser`, `ThemeUser`, `AssetUser`), each with diverging fallback logic and redundant claims extraction.
5. **Undefined Inter-Slice Boundaries:** Crates query other domains through `cms-db` rather than importing typed domain APIs directly.

---

## 2. Architectural Objectives

1. **True Domain Sovereignty:** Each domain crate owns its domain models, compile-time SQLx queries, business logic, validation, and HTTP endpoints.
2. **Zero Shims & Zero Duplication:** Completely eliminate `pub use cms_db::...` shims, duplicate query implementations, and redundant authentication extractors.
3. **Infrastructure Isolation for `cms-db`:** Reduce `cms-db` strictly to database infrastructure: connection pool, migration runner, RLS context, and cross-cutting audit primitives.
4. **Dissolution of `cms-entity`:** Universal primitives (e.g. `PaginatedResponse`) move to `cms-server-core::pagination`. Domain models live inside their respective domain crates.
5. **Unified Authentication Extractor (`cms-auth::AuthUser`):** A single, canonical Axum `FromRequestParts` extractor imported by all slices.
6. **Clean Directed Acyclic Graph (DAG):** Cross-slice access happens via direct crate dependencies (e.g., `cms-content` imports `cms-projects`), with zero cyclic dependencies.

---

## 3. Crate Responsibilities & Boundaries

### 3.1 Infrastructure & Cross-Cutting Crates

| Crate | Responsibilities | Does NOT Contain |
|---|---|---|
| `cms-db` | Connection pooling (`create_pool`), migrations (`sqlx::migrate!()`), RLS write/read context (`set_write_context`), atomic sequences (`next_sequence`), immutable audit logging (`audit.rs`). | Domain queries, domain tables, business logic. |
| `cms-server-core` | Unified `AppState`, HTTP tracing, enterprise middleware (rate limit, CSRF, security headers, locale), pagination primitives (`PaginatedResponse`). | Domain routes or entity models. |
| `cms-auth` | JWT issuance/verification, password hashing (Argon2), session management, MFA/TOTP, OAuth, **canonical `AuthUser` extractor**. | Slice-specific business logic. |
| `cms-authz` | Zanzibar ReBAC engine (`backend_pg.rs`), permission gates (`require_project_access`, `require_parent_access`). | Direct HTTP endpoints. |
| `cms-error` | Unified `AppError` and HTTP status mapping. | Domain models. |
| `cms-infra` | Storage service, email mailer/renderer, queue client, config loading. | Domain entities. |

### 3.2 Sovereign Vertical Slices

Each vertical slice is an autonomous domain module structured around **Domain Aggregate Modules**:

| Vertical Slice | Owned Entities & Tables | Public API Provided |
|---|---|---|
| `cms-projects` | `Project`, `ProjectSettings`, `ProjectAddon`, `ProjectMember`, custom domains, reader access. | `ProjectQueries`, `ProjectMemberQueries`, `DomainQueries`, `project_router`. |
| `cms-content` | `Page`, `Branch`, `Language`, `Comment`. | `PageQueries`, `BranchQueries`, `LanguageQueries`, `CommentQueries`, `content_router`. |
| `cms-deployments` | `Deployment`, deployment logs, build artifacts, snapshots. | `DeploymentService`, `process_deployment_job`, `deployments_router`. |
| `cms-git` | Git repositories, provider credentials (GitHub/GitLab), sync jobs, conflicts. | `GitSyncEngine`, `process_git_job`, `git_router`. |
| `cms-integrations` | Outbound webhooks (Slack, Discord, Teams), SSRF validator. | `IntegrationService`, `integrations_router`. |
| `cms-themes` | Project & global themes, CSS variable compiler. | `ThemeService`, `themes_router`. |
| `cms-assets` | Asset metadata, presigned upload URLs. | `AssetService`, `assets_router`. |
| `cms-notifications`| In-app notifications, thread-safe WebSocket `NotificationHub`. | `NotificationService`, `NotificationHub`, `notifications_router`. |
| `cms-invitations` | Workspace invitations, token hashing, email outbox events. | `InvitationService`, `invitations_router`. |
| `cms-onboarding` | System setup status, email verification tokens. | `OnboardingService`, `onboarding_router`. |
| `cms-reports` | Export runs, background PDF/ePub generation. | `ReportService`, `reports_router`. |
| `cms-workspaces` | Workspace configuration and members. | `WorkspaceService`, `workspaces_router`. |
| `cms-admin` | Platform moderation takedowns, system metrics, platform telemetry. | `AdminService`, `admin_router`. |
| `cms-public` | Reader unauthenticated JSON API, RSS feeds, sitemaps. | `public_router`. |
| `cms-analytics` | Workspace page views, search analytics. | `AnalyticsService`, `analytics_router`. |
| `cms-api-specs` | Uploaded OpenAPI documents. | `ApiSpecsService`, `api_specs_router`. |
| `cms-mcp` | RMCP Model Context Protocol AI tools. | `mcp_router`. |
| `cms-search` | Tantivy BM25 + Lindera Japanese + FastEmbed semantic search. | `SearchEngine`, `search_router`. |

---

## 4. Internal Domain Aggregate Architecture

Inside each vertical slice crate (e.g. `crates/cms-content/`), code is organized by domain aggregates rather than artificial technical layers:

```
crates/cms-content/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── page.rs
    ├── branch.rs
    ├── language.rs
    └── comment.rs
```

Inside each aggregate module (e.g., `page.rs`):
1. **Domain Models & DTOs:** `Page`, `PageRow`, `CreatePageRequest`, `UpdatePageRequest`, `PageResponse`.
2. **Database Queries:** Internal `PageQueries` issuing compile-time verified SQLx statements (`sqlx::query_as!<_, PageRow>(...)`).
3. **Business Invariants:** Tree hierarchy validation, path materialization, cycle detection.
4. **Axum Handlers & Router:** `create_page_handler`, `get_page_handler`, `page_subrouter`.

---

## 5. Unified Authentication Pattern

All 7 duplicate slice extractors (`ContentUser`, `ProjectUser`, `DeploymentUser`, `GitUser`, `IntegrationUser`, `ThemeUser`, `AssetUser`) are deleted.

All vertical slices import the canonical extractor from `cms-auth`:
```rust
use cms_auth::AuthUser;

pub async fn create_page_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, branch_id)): Path<(String, String)>,
    Json(payload): Json<CreatePageRequest>,
) -> Result<Json<PageResponse>, AppError> {
    // 1. Authorize via Zanzibar ReBAC gate
    cms_authz::require_project_access(&state.authz, auth.id, &project_id, "editor").await?;
    
    // 2. Execute domain logic
    let page = PageService::create_page(&state.pool, &auth.id.to_string(), &project_id, &branch_id, payload).await?;
    Ok(Json(page))
}
```

---

## 6. Inter-Slice Dependency Graph (DAG)

Inter-slice interactions follow a strict Directed Acyclic Graph:

```
apps/api (Composition Root)
  │
  ├── cms-public ──────┐
  ├── cms-deployments ──┼──> cms-content ──> cms-projects ──┐
  ├── cms-git ──────────┤                                    │
  ├── cms-sites ────────┘                                    │
  │                                                          │
  ├── cms-assets, cms-themes, cms-integrations, ... ─────────┼──> cms-authz ──> cms-auth
  │                                                          │        │             │
  └── cms-scheduler, cms-worker, cms-outbox ─────────────────┴────────┴─────────────┴──> cms-server-core
                                                                                               │
                                                                                          cms-db (pool, rls, migrations)
```

Rules:
- Downstream domain crates may depend on upstream domain crates (e.g. `cms-content` -> `cms-projects`).
- Upstream domain crates never depend on downstream domain crates.
- Circular dependencies are strictly prohibited.

---

## 7. Migration Sequence

1. **Phase 1: Foundation & Auth Unification**
   - Move `PaginatedResponse` into `cms-server-core::pagination`.
   - Implement `AuthUser` in `cms-auth`.
   - Update `cms-auth` tests.
2. **Phase 2: Projects Slice (`cms-projects`)**
   - Consolidate all project, member, domain, and reader access queries/models into `cms-projects`.
   - Delete duplicate `ProjectQueries` from `cms-db`.
   - Replace `ProjectUser` with `cms_auth::AuthUser`.
3. **Phase 3: Content Slice (`cms-content`)**
   - Move page, branch, language, and comment queries/models into `cms-content`.
   - Delete shims in `cms-content/src/queries/` and `cms-content/src/models/`.
   - Depend directly on `cms-projects` for project and language validation.
   - Replace `ContentUser` with `cms_auth::AuthUser`.
4. **Phase 4: Secondary Slices**
   - Migrate `cms-deployments`, `cms-git`, `cms-assets`, `cms-themes`, `cms-integrations`, `cms-notifications`, `cms-invitations`, `cms-onboarding`, `cms-reports`, `cms-admin`, `cms-analytics`, `cms-api-specs`.
   - Remove duplicate extractors and transfer domain queries out of `cms-db`.
5. **Phase 5: Purge `cms-db` & Dissolve `cms-entity`**
   - Remove all domain query files from `cms-db`.
   - Dissolve `cms-entity` and update workspace root `Cargo.toml`.
6. **Phase 6: Verification & Gate Checks**
   - Verify compile-time queries: `SQLX_OFFLINE=true cargo check --workspace`.
   - Verify workspace test suite: `cargo test --workspace`.
   - Ensure zero compiler warnings and zero dead code.
