# Phase 3 to Phase 5 Implementation Plan: Complete Vertical Slices and Eliminate `cms-biz` and `cms-api`

## Goal
Complete the transformation of `cms-rs-4` into a true modular monolith (aligned with MedBrains architecture). Encapsulate all remaining business logic and API endpoints into their cohesive vertical slices and specialized infrastructure crates, assemble the application directly in the composition root (`apps/api`), and completely delete the `cms-biz` and `cms-api` crates.

---

## Architecture Principles
1. **Vertical Slice Autonomy**: Each domain slice encapsulates its models, SQLx queries, business logic services, Axum handlers, and subrouter.
2. **Decoupled Services**: Services accept `pool: &PgPool` directly or domain dependencies, with zero dependency on a monolithic `BizContext`.
3. **No Direct Crate Cycles**: Vertical slices depend on `cms-server-core`, `cms-server-services`, `cms-db`, `cms-entity`, `cms-error`, `cms-auth`, `cms-authz`.
4. **Composition Root Assembly**: `apps/api` assembles the final Axum router by mounting each slice router directly (`/projects`, `/pages`, `/assets`, `/deployments`, `/git`, etc.).
5. **Clean Deprecation**: Delete `cms-biz` and `cms-api` completely from workspace members and filesystem once all domains are migrated.

---

## Phase Breakdown

### Phase 3: Infrastructure & Direct Subsystems Consolidation
Migrate domain services and API routers for systems that already have dedicated infrastructure crates:
- **Task 1: Search Subsystem (`crates/cms-search`)**
  - Move `SearchService` (from `cms-biz/src/search.rs`) to `cms-search::services::SearchService`.
  - Add `cms-search::handlers` and `pub fn router(state: Arc<AppState>) -> Router`.
  - Wire tests in `crates/cms-search/tests/`.
  - Delegate `crates/cms-api/src/search/` to `cms-search`.

- **Task 2: Analytics Subsystem (`crates/cms-analytics`)**
  - Move `AnalyticsService` (from `cms-biz/src/analytics.rs`) to `cms-analytics::services::AnalyticsService`.
  - Add `cms-analytics::handlers` and `pub fn router(state: Arc<AppState>) -> Router`.
  - Wire tests in `crates/cms-analytics/tests/`.
  - Delegate `crates/cms-api/src/analytics/` to `cms-analytics`.

- **Task 3: MCP Subsystem (`crates/cms-mcp`)**
  - Move `McpService` (from `cms-biz/src/mcp.rs`) to `cms-mcp::services::McpService`.
  - Add `cms-mcp::handlers` and `pub fn router(state: Arc<AppState>) -> Router`.
  - Wire tests in `crates/cms-mcp/tests/`.
  - Delegate `crates/cms-api/src/mcp/` to `cms-mcp`.

---

### Phase 4: Remaining Domain Vertical Slices Extraction
Extract dedicated high-cohesion vertical slice crates for the remaining modules:
- **Task 4: Deployment Vertical Slice (`crates/cms-deployments`)**
  - Scaffolding & workspace registration (`crates/cms-deployments`).
  - Move deployment queries, models, `DeploymentService`, Axum handlers, and router.
  - Contract & serde tests.
  - Delegate `crates/cms-api/src/deployment/` to `cms-deployments`.

- **Task 5: Git Integration Vertical Slice (`crates/cms-git`)**
  - Scaffolding & workspace registration (`crates/cms-git`).
  - Move git queries, models, `GitService`, Axum handlers, and router.
  - Contract & serde tests.
  - Delegate `crates/cms-api/src/git/` to `cms-git`.

- **Task 6: Integrations Vertical Slice (`crates/cms-integrations`)**
  - Scaffolding & workspace registration (`crates/cms-integrations`).
  - Move integration queries, models, `IntegrationService`, Axum handlers, and router.
  - Contract & serde tests.
  - Delegate `crates/cms-api/src/integration/` to `cms-integrations`.

- **Task 7: Themes Vertical Slice (`crates/cms-themes`)**
  - Scaffolding & workspace registration (`crates/cms-themes`).
  - Move theme queries, models, `ThemeService`, Axum handlers, and router.
  - Contract & serde tests.
  - Delegate `crates/cms-api/src/theme/` to `cms-themes`.

---

### Phase 5: Auth, Admin, OpenAPI & Final Elimination of `cms-biz` and `cms-api`
- [x] **Task 8: Auth & Identity Slice Consolidation (`crates/cms-auth`)**
  - Moved `AuthService`, user queries, and auth router into `crates/cms-auth`.
  - Migrated user/session/auth handlers.

- [x] **Task 9: OpenAPI Documentation & Platform Event Slice**
  - Extracted OpenAPI generator into `crates/cms-openapi`.
  - Extracted platform events into `crates/cms-platform-events`.
  - Extracted workspace configuration into `crates/cms-workspaces`.
  - Extracted administrative operations into `crates/cms-admin`.
  - Extracted public reader API into `crates/cms-public`.

- [x] **Task 10: Composition Root Router Assembly (`apps/api`)**
  - Implemented full router composition in `apps/api/router.rs` and `apps/api/lib.rs` mounting all vertical slice routers directly.
  - Updated `apps/api/main.rs` to use `cms_server::api_router`.
  - Moved all end-to-end integration tests to `apps/api/tests/`.

- [x] **Task 11: Remove `cms-biz` and `cms-api`**
  - Removed `cms-biz` and `cms-api` from root `Cargo.toml`.
  - Removed all crate dependencies on `cms-biz` and `cms-api`.
  - Deleted directories `crates/cms-biz` and `crates/cms-api`.

- [ ] **Task 12: Full Monorepo Verification & Documentation**
  - Run `cargo check --workspace -j 2`.
  - Run all workspace tests: `cargo test --workspace -j 2`.
  - Run clippy check: `cargo clippy --all-targets --all-features -j 2`.
  - Update `AGENTS.md` and architecture records.
