# Vertical Slices & Shared Services SDD Progress

Baseline: 2adfece
Branch: feat/vertical-slices-subsystems
Plan: docs/superpowers/plans/2026-10-12-vertical-slices-modular-monolith.md

## Tasks
- [x] Task 1: Create `crates/cms-server-services` Crate
- [x] Task 2: Extract `crates/cms-invitations` Vertical Slice Crate
- [x] Task 3: Extract `crates/cms-onboarding` Vertical Slice Crate
- [x] Task 4: Extract `crates/cms-reports` Vertical Slice Crate
- [x] Task 5: End-to-End Verification & Documentation Update

## Phase 1: Content Core Vertical Slice (`cms-content`)
Plan: `docs/superpowers/plans/2026-10-12-phase1-cms-content-vertical-slice.md`
- [x] Task 1: Crate Scaffolding & Workspace Registration (`crates/cms-content`)
- [x] Task 2: Page Domain Vertical Slice (models, queries, service, handler, tests)
- [x] Task 3: Branch Domain Vertical Slice (models, queries, service, handler, tests)
- [x] Task 4: Language Domain Vertical Slice (models, queries, service, handler, tests)
- [x] Task 5: Comment Domain Vertical Slice & Composite Router
- [x] Task 6: Wire into `cms-api` and delegate routers
- [x] Task 7: Full Workspace Verification, Clippy & Documentation Update

## Phase 2: Asset & Project Vertical Slices (`cms-assets` & `cms-projects`)
Plan: `docs/superpowers/plans/2026-10-12-phase2-cms-assets-and-projects-vertical-slices.md`
- [x] Task 1: Crate Scaffolding & Workspace Registration (`crates/cms-assets`)
- [x] Task 2: Models and SQLx Queries (`cms-assets`)
- [x] Task 3: Decoupled Service (`cms-assets`)
- [x] Task 4: Axum Handlers & Routers (`cms-assets`)
- [x] Task 5: Contract & Serde Tests (`cms-assets`)
- [x] Task 6: Wire `cms-assets` into `cms-api` and verify
- [x] Task 7: Crate Scaffolding & Workspace Registration (`crates/cms-projects`)
- [x] Task 8: Models and SQLx Queries (`cms-projects`)
- [x] Task 9: Decoupled Services (`cms-projects`)
- [x] Task 10: Axum Handlers & Routers (`cms-projects`)
- [x] Task 11: Contract & Serde Tests (`cms-projects`)
- [x] Task 12: Wire `cms-projects` into `cms-api` and full workspace verification

## Phase 4: Remaining Domain Vertical Slices Extraction
Plan: `docs/superpowers/plans/2026-10-12-phase3-to-phase5-vertical-slices-and-eliminate-biz-api.md`
- [x] Task 4: Deployment Vertical Slice (`crates/cms-deployments`)
  - Scaffolding & workspace registration (`crates/cms-deployments`)
  - Models, queries, `DeploymentService`, Axum handlers, router, and contract tests
  - Decoupled `process_deployment_job` in `cms-worker`
  - Delegated `crates/cms-api/src/deployment/` to `cms-deployments`
- [x] Task 5: Git Integration Vertical Slice (`crates/cms-git`)
  - Scaffolding & workspace registration (`crates/cms-git`)
  - `GitService`, `GitAuth` extractor, Axum handlers, router, and contract tests
  - Decoupled `process_git_job` in `cms-worker`
  - Delegated `crates/cms-api/src/git/` to `cms-git`
- [x] Task 6: Integrations Vertical Slice (`crates/cms-integrations`)
  - Scaffolding & workspace registration (`crates/cms-integrations`)
  - `IntegrationService`, SSRF protection, `IntegrationAuth` extractor, Axum handlers, router, and contract tests
  - Delegated `crates/cms-api/src/integration/` to `cms-integrations`
- [x] Task 7: Themes Vertical Slice (`crates/cms-themes`)
  - Scaffolding & workspace registration (`crates/cms-themes`)
  - `ThemeService`, `ThemeAuth` extractor, Axum handlers, router, and contract tests
  - Delegated `crates/cms-api/src/theme/` to `cms-themes`
- [x] Cleanup: Removed dead `git.rs`, `integration.rs`, `theme.rs` from `cms-biz`

## Phase 5: Auth, Admin, OpenAPI & Complete Elimination of `cms-biz` and `cms-api`
Plan: `docs/superpowers/plans/2026-10-12-phase3-to-phase5-vertical-slices-and-eliminate-biz-api.md`
- [x] Task 8: Auth & Identity Slice Consolidation (`crates/cms-auth`)
  - Models, user queries, `AuthService`, session/JWT services, and auth router
- [x] Task 9: Autonomous Vertical Slices Scaffolding & Migration
  - `crates/cms-platform-events`: System health & platform telemetry vertical slice
  - `crates/cms-openapi`: OpenAPI spec generation & docs vertical slice
  - `crates/cms-workspaces`: Workspace configuration & member lifecycle vertical slice
  - `crates/cms-admin`: Administrative operations parity vertical slice
  - `crates/cms-public`: Public reader API vertical slice
- [x] Task 10: Composition Root Router Assembly (`apps/api`)
  - Composed production router in `apps/api/router.rs` and `apps/api/lib.rs`
  - Wired into `apps/api/main.rs` with `cms_server::api_router`
  - Moved full suite of end-to-end integration tests to `apps/api/tests/`
- [x] Task 11: Complete Elimination of `cms-biz` and `cms-api`
  - Purged from workspace root `Cargo.toml`
  - Deleted `crates/cms-biz` and `crates/cms-api` directories
- [ ] Task 12: Full Workspace Verification, Clippy & Documentation Update

