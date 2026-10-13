# CMS-RS-4 Architecture Map

> **Status:** Phase A complete — derived from source inspection on 2026-10-13.
> All facts below are verified against the actual source. Assumptions are labelled.

---

## 1. Application Entry Points & Runtime Boundaries

| Binary | Path | Role |
|---|---|---|
| `cms-server` | `apps/api/main.rs` | Primary HTTP server + embedded workers |
| `cms-seed` CLI | `crates/cms-seed/` | Dev fixture generator |

`main.rs` initialises observability, loads `Config`, builds `AppState`, starts `cms-worker` background consumers (same process), mounts the full router, then starts `axum::serve` with graceful shutdown.

---

## 2. Crate/Module Responsibilities & Dependency Direction

```
apps/api (composition root)
├── cms-server-core        (AppState, middleware, pagination, health)
├── cms-auth               (JWT, sessions, API keys, MFA, OAuth, extractors)
├── cms-authz              (Zanzibar ReBAC — PgAuthzBackend, TestAuthzBackend)
├── cms-db                 (SQLx query structs, migrations, audit, context)
├── cms-entity             (Pure domain types — zero internal deps)
├── cms-error              (AppError, HTTP error mapping)
├── cms-infra              (Config, Storage, Mailer, JobQueue, EmailRenderer)
│
├── cms-content            (pages, branches, languages, comments)
├── cms-projects           (project workspace, members, domains, reader access)
├── cms-deployments        (deployment lifecycle, site building)
├── cms-git                (Git connections, GitHub/GitLab sync)
├── cms-integrations       (webhooks -- Slack, Discord, Teams, SSRF guard)
├── cms-themes             (project/global themes)
├── cms-assets             (asset upload, storage presigning)
├── cms-notifications      (WebSocket hub, in-app notifications)
├── cms-invitations        (workspace invitations, email dispatch via outbox)
├── cms-onboarding         (setup status, email verification)
├── cms-reports            (export lifecycle, background PDF/ePub)
├── cms-workspaces         (workspace config, workspace members)
├── cms-admin              (platform-admin ops, metrics, platform events)
├── cms-public             (unauthenticated reader API, SEO, RSS, sitemap)
├── cms-analytics          (page views, search metrics)
├── cms-api-specs          (stored OpenAPI docs)
├── cms-mcp                (rmcp MCP server endpoints)
├── cms-search             (Tantivy BM25 + Lindera Japanese + FastEmbed RAG)
│
├── cms-outbox             (transactional outbox worker)
├── cms-scheduler          (distributed cron engine)
├── cms-worker             (background job processor)
└── cms-sites              (SPA server + static reader site generator)
```

**Dependency direction:** `apps/api -> all vertical slices -> cms-db -> cms-entity`. No cycles observed.

---

## 3. Request Lifecycle & Error Propagation

```
TcpListener
  └─ axum::serve (with_graceful_shutdown)
       └─ Router
            ├─ tower_http::TimeoutLayer (30s, returns 408)
            ├─ tower_http::CatchPanicLayer
            ├─ tower_http::CompressionLayer
            ├─ tower_http::CorsLayer  (configurable allow-list)
            ├─ tower_http::TraceLayer
            ├─ axum::Extension(Arc<AppState>)
            ├─ locale_scoping_middleware
            ├─ RateLimiter (sliding window, governor)
            ├─ SecurityHeadersMiddleware
            └─ ObservabilityLayer (tracing, metrics)
                 └─ /api -> api_router
                       └─ /auth, /projects, /pages, ...
                            └─ AuthExtractor (cookie -> JWT -> API-key)
                                 └─ Handler -> AppError -> (StatusCode, JSON)
```

`AppError` variants map to HTTP status codes in `cms-error`. Handler returns `Result<_, AppError>`.

---

## 4. Authentication Flow

1. `AuthExtractor::from_request_parts` is a `FromRequestParts` extractor (no middleware; runs per-handler).
2. Priority order: `access_token` cookie -> legacy `session_token` cookie -> `Authorization: Bearer` JWT -> `Authorization: Bearer` API-key-pattern -> `X-API-Key` header.
3. Each path calls into `cms-db` queries and returns `(UserResponse, Claims)`.
4. Several code paths (session cookie path, Bearer API-key path, X-API-Key path, OAuth path) fall back to `Uuid::parse_str(&user.id).unwrap_or_else(|_| Uuid::nil())` -- see ISSUES.md P0-001.

Alongside `AuthExtractor`, **each vertical slice defines its own independent auth extractor** (`DeploymentUser`, `ContentUser`, `ProjectUser`, `GitUser`, `IntegrationUser`, `ThemeUser`, `AssetUser`). These duplicate each other and differ in bypass logic -- see ISSUES.md P1-003.

---

## 5. Authorization Flow

- `cms-authz::PgAuthzBackend::check()` implements Zanzibar `RelationTupleQueries::check` (single indexed SQL query).
- `is_bypass` flag in `AuthzContext` causes `check()` to unconditionally return `true`.
- **Bypass roles in `cms-auth::middleware`:** `super_admin`, `owner`, `admin` -- admin/owner bypass should only apply to workspace-level operations.
- **Bypass roles in vertical-slice extractors:** each crate defines its own bypass list (`super_admin`, `owner`, `admin`); these are independently maintained and diverge.
- `require_project_access` / `require_parent_access` in `cms-authz::gate` are the canonical authorization check functions. They return `NotFound` (not `Forbidden`) to prevent existence oracle attacks.

---

## 6. Data Flow

Transport -> Handler -> (cms-db query functions or vertical-slice service) -> PostgreSQL.

No ORM. All queries hand-written SQLx. Offline metadata committed to `.sqlx/`.

---

## 7. Transaction Boundaries & Consistency

- `cms_db::set_write_context(&mut tx, ...)` must be called after `pool.begin()` to set RLS tenant context.
- Outbox events must be queued inside the same transaction as business writes (`cms_outbox::queue_in_tx`).
- Session creation in `login_handler` is NOT transactional -- result is silently discarded (`let _ = SessionQueries::create(...).await`).

---

## 8. Background Tasks, Scheduling & Shutdown

- `OutboxWorker`: polls `outbox_events` with `FOR UPDATE SKIP LOCKED`, batch of 32, exponential jitter backoff, DLQ after 10 attempts, reaper for abandoned locks. Shutdown via `watch::Receiver<bool>`.
- `cms-worker` (`start_consumers_with_shutdown`): processes typed `JobType` payloads (export, deployment, Git sync, etc.).
- `cms-scheduler`: distributed cron with `FOR UPDATE SKIP LOCKED`, pre-registered maintenance jobs.
- Workers share the HTTP process (same `main.rs`). Graceful shutdown: HTTP server drains, then `worker_shutdown_tx.send(true)`, then `join` all handles.

---

## 9. Configuration

`cms-infra::config::Config` loaded via `figment` (TOML + env vars).

Validation runs on `AppState::from_config`. Migrations run at startup.

---

## 10. External Integrations

| Service | Crate | Notes |
|---|---|---|
| PostgreSQL 14+ | cms-db | Primary store |
| Tantivy (disk) | cms-search | BM25 FTS |
| FastEmbed ONNX | cms-search | Vector/semantic search |
| S3 / local disk | cms-infra storage | Asset storage |
| SMTP / Lettre | cms-infra mailer | Transactional email |
| GitHub / GitLab | cms-git | Git sync webhooks |
| Slack/Discord/Teams | cms-integrations | Outbound webhooks |
| MCP (rmcp) | cms-mcp | AI agent integration |

---

## 11. Confirmed Conflicts & Architectural Problems

| # | Area | Problem | Status |
|---|---|---|---|
| 1 | Auth middleware | `Uuid::parse_str(...).unwrap_or_else(nil)` -- silent nil UUID | Open |
| 2 | Auth middleware | `is_bypass` granted to `owner` + `admin` (too broad) | Open |
| 3 | Auth extractors | 7+ independently duplicated auth extractors per vertical slice | **RESOLVED** (Migrated to canonical `cms_auth::AuthUser`) |
| 4 | `cms-db` | `DomainQueries` defined in both `domain.rs` and `deployment.rs` | **RESOLVED** (Relocated to sovereign `cms-projects::DomainQueries`) |
| 5 | Login handler | `let _ = SessionQueries::create(...)` -- session failure silently ignored | Open |
| 6 | `cms-db` lib | `#![allow(clippy::unnecessary_unwrap)]` masks real defects | Open |
| 7 | Compile | `axum::http::HeaderValue` unnecessary qualification | **RESOLVED** |
