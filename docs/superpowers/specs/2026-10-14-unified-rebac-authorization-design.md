# Unified ReBAC Authorization Architecture & Gating Specification

- **Date:** 2026-10-14
- **Status:** Approved
- **Scope:** `cms-rs-4` backend modular monolith (`crates/cms-authz`, `crates/cms-db`, `crates/cms-auth`, and vertical slices: `cms-themes`, `cms-git`, `cms-integrations`, `cms-deployments`, `cms-assets`, `cms-analytics`, `cms-content`, `cms-projects`).

---

## 1. Problem Statement

Across `cms-rs-4`, ReBAC authorization previously suffered from two core structural issues:
1. **Ad-hoc & Fragmented Implementations ("Not Unified"):**
   - While `cms-authz::gate` introduced `ParentLink` (`Page`, `Branch`, `Deployment`, `ApiKey`), multiple vertical slices (`cms-themes`, `cms-git`, `cms-integrations`, `cms-deployments`, `cms-assets`) bypassed it and implemented their own bespoke `authorize_*_access` helper functions in their respective `handlers.rs`.
   - Each bespoke helper repeated database lookups, string-to-UUID parsing, and error-masking (`NotFound | Forbidden -> NotFound`) to prevent existence oracle leaks.
2. **Fragile Handlers ("Not in the Right Place"):**
   - Authorization was called imperatively at the top of handler bodies. Forgetting an `authorize_*` call left endpoints exposed (as observed in earlier P0 deployment authorization defects).

---

## 2. Target Architecture

The unified authorization framework operates on a **2-Tier Model**:

```
+---------------------------------------------------------------------------------+
|                               HTTP REQUEST                                      |
+---------------------------------------+-----------------------------------------+
                                        |
                 +----------------------+----------------------+
                 |                                             |
   [Direct Project Route]                             [Child Entity Route]
   /projects/:project_id/...                          /themes/:id, /deployments/:id, ...
                 |                                             |
                 v                                             v
        +------------------+                       +-----------------------+
        |  Axum Extractor  |                       | Handler calls unified |
        |  ProjectViewer / |                       | require_parent_access |
        |  ProjectEditor   |                       +-----------+-----------+
        +--------+---------+                                   |
                 |                                             v
                 |                              +------------------------------+
                 |                              |     cms_authz ParentLink     |
                 |                              | (Theme, Deployment, Git, …)  |
                 |                              +--------------+---------------+
                 |                                             |
                 |                                             v
                 |                              +------------------------------+
                 |                              | single indexed lookup in DB: |
                 |                              |  resolve_*_project(pool, id) |
                 |                              +--------------+---------------+
                 |                                             |
                 v                                             v
   +-----------------------------------------------------------------------------+
   |            cms_authz::gate::require_project_access(project_id, relation)    |
   |           Evaluates Zanzibar ReBAC -> Returns 404 on any denial             |
   +-----------------------------------------------------------------------------+
```

### Tier 1: Direct Project Routes (`ProjectViewer` & `ProjectEditor`)
For routes structured under `/projects/:project_id/...` or `/projects/:id/...`:
- Define custom Axum extractors in `crates/cms-auth/src/extractors.rs`:
  - `ProjectViewer`: Authenticates user and verifies `Relation::Viewer` on `project_id`.
  - `ProjectEditor`: Authenticates user and verifies `Relation::Editor` on `project_id`.
- Handlers declare `viewer: ProjectViewer` or `editor: ProjectEditor` as function parameters.
- **Security Guarantee:** Compile-time and framework-level enforcement. If authorization fails, the handler is never entered, and `404 Not Found` is returned.

### Tier 2: Child Entity Routes (`ParentLink` & `require_parent_access`)
For routes identifying child resources directly (`/themes/:id`, `/git/connections/:id`, `/integrations/:id`, `/deployments/:id`, `/assets/:id`, `/pages/:id`, `/branches/:id`):
- Extend `cms_authz::gate::ParentLink` to encompass all child entities:
  - `Page(Uuid)`
  - `Branch(Uuid)`
  - `Deployment(Uuid)`
  - `ApiKey(Uuid)`
  - `Theme(Uuid)`
  - `GitConnection(Uuid)`
  - `Integration(Uuid)`
  - `Asset(Uuid)`
- Provide single-query database resolution in `cms_db::authz_queries`:
  - `resolve_page_project`
  - `resolve_branch_project`
  - `resolve_deployment_project`
  - `resolve_api_key_project`
  - `resolve_theme_project`
  - `resolve_git_connection_project`
  - `resolve_integration_project`
  - `resolve_asset_project`
- Centralize error collapsing in `cms_authz::gate::require_parent_access`:
  - Resolves parent `project_id`.
  - Checks Zanzibar ReBAC grant.
  - Automatically collapses any `NotFound` or `Forbidden` into `AppError::NotFound("Resource not found")` without leaking resource existence.

---

## 3. Detailed Component Specifications

### 3.1 `cms-db` (`crates/cms-db/src/authz_queries.rs`)
Add lookup functions using parameterized compile-time/indexed SQL:
- `resolve_theme_project(pool: &PgPool, theme_id: Uuid) -> Result<Option<Uuid>, AppError>`
  - Resolves from `"ProjectSettings"` or theme table.
- `resolve_git_connection_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError>`
  - Resolves from `"GitConnection"`.
- `resolve_integration_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError>`
  - Resolves from `"ProjectIntegration"`.
- `resolve_asset_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError>`
  - Resolves from `"Asset"`.

### 3.2 `cms-authz` (`crates/cms-authz/src/gate.rs`)
- Expand `ParentLink` enum.
- Dispatch each enum variant in `resolve_parent_project` to `cms_db::*`.
- Ensure `require_parent_access` collapses denials into `AppError::NotFound("Resource not found".to_string())`.

### 3.3 `cms-auth` (`crates/cms-auth/src/extractors.rs`)
Implement `FromRequestParts` for `ProjectViewer` and `ProjectEditor`:
- Extract `Arc<AppState>` from extensions.
- Extract `AuthUser` via `AuthUser::from_request_parts`.
- Parse project ID from path parameters (`:project_id` or `:id`).
- Invoke `require_project_access(state.authz_backend(), &auth.authz_context(), project_id, relation)`.
- Re-export `ProjectViewer` and `ProjectEditor` from `cms_auth`.

### 3.4 Vertical Slice Migration & Refactoring
- **`cms-themes`**:
  - Direct routes: `list_themes_handler` (`ProjectViewer`), `create_theme_handler` (`ProjectEditor`).
  - Child routes (`get_theme_handler`, `update_theme_handler`, `delete_theme_handler`): use `ParentLink::Theme`.
  - Remove `authorize_theme_access` and `authorize_theme_project_access`.
- **`cms-git`**:
  - Direct routes: `list_connections` (`ProjectViewer`), `create_connection` (`ProjectEditor`).
  - Child routes: use `ParentLink::GitConnection`.
  - Remove `authorize_git_connection_access`.
- **`cms-integrations`**:
  - Direct routes: `list_integrations` (`ProjectViewer`), `create_integration` (`ProjectEditor`).
  - Child routes: use `ParentLink::Integration`.
  - Remove `authorize_integration_access`.
- **`cms-deployments`**:
  - Direct routes: `list_deployments` (`ProjectViewer`), `publish_handler` (`ProjectEditor`).
  - Child routes (`get_deployment`, `cancel_deployment`, `retry_deployment`, `logs_deployment`): use `ParentLink::Deployment`.
  - Remove `authorize_deployment_access`.
- **`cms-assets`**:
  - Direct routes: `list_assets` (`ProjectViewer`), `upload_asset` (`ProjectEditor`).
  - Child routes: use `ParentLink::Asset`.
- **`cms-analytics`**:
  - Direct routes: `get_overview`, `get_page_views`, etc. (`ProjectViewer`).

---

## 4. Verification and Testing

1. **Unit & Contract Tests**:
   - Verify `ParentLink` resolutions for each entity type in `cms-authz::gate::tests`.
   - Verify `ProjectViewer` and `ProjectEditor` rejection with `404 Not Found` when unauthorized or nonexistent.
2. **Integration Tests**:
   - Run `cargo test -p cms-themes -p cms-git -p cms-integrations -p cms-deployments -p cms-assets -p cms-analytics -p cms-authz`.
   - Run full API test suite `apps/api/tests/test_17_project_themes.rs`, `test_14_deployments.rs`, `test_15_git.rs`, `test_16_integrations.rs`.
3. **Clippy & Lints**:
   - Run `cargo clippy --workspace --all-targets -- -D warnings`.
