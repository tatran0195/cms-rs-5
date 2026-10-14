# Unified ReBAC Authorization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify Zanzibar ReBAC authorization across all CMS vertical slices by extending `cms-authz::gate::ParentLink` with database resolvers in `cms-db`, introducing typed Axum extractors (`ProjectViewer`, `ProjectEditor`) in `cms-auth`, and eliminating ad-hoc authorization boilerplate.

**Architecture:** 2-tier authorization pattern: Direct project routes (`/projects/:project_id/...`) extract `ProjectViewer`/`ProjectEditor` via Axum's `FromRequestParts` to enforce ReBAC before handler execution; Child entity routes (`/themes/:id`, `/deployments/:id`, `/git/:id`, `/integrations/:id`, `/assets/:id`) resolve parent project IDs via unified `cms_authz::gate::ParentLink` and `require_parent_access`. Denials collapse to `404 Not Found` without leaking resource existence.

**Tech Stack:** Rust 2024, Axum 0.8, SQLx, Zanzibar ReBAC (`cms-authz`), `cms-db`, `cms-auth`, `cms-error`.

## Global Constraints

- Edition 2024, minimum Rust version 1.85.
- `unsafe` code is strictly forbidden (`forbid(unsafe_code)`).
- Strict clippy: `clippy::all` denied, no `unwrap`, `expect`, or `panic`.
- Never leak resource existence: unauthorized access or missing resources must return `AppError::NotFound`.
- Keep changes zero-allocation / minimal round-trip where possible.

---

### Task 1: Add Parent Resolution Queries in `cms-db`

**Files:**
- Modify: `crates/cms-db/src/authz_queries.rs`
- Modify: `crates/cms-db/src/lib.rs`

**Interfaces:**
- Produces:
  - `pub async fn resolve_theme_project(pool: &PgPool, theme_id: Uuid) -> Result<Option<Uuid>, AppError>`
  - `pub async fn resolve_git_connection_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError>`
  - `pub async fn resolve_integration_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError>`
  - `pub async fn resolve_asset_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError>`

- [ ] **Step 1: Implement the resolution queries in `authz_queries.rs`**

```rust
/// Resolve the parent project for a theme (from ProjectSettings where theme is configured).
pub async fn resolve_theme_project(
    pool: &PgPool,
    theme_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT project_id::uuid FROM "ProjectSettings" WHERE project_id = $1"#
    )
    .bind(theme_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for a git connection.
pub async fn resolve_git_connection_project(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT project_id::uuid FROM "GitConnection" WHERE id = $1"#
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for an integration.
pub async fn resolve_integration_project(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT project_id::uuid FROM "ProjectIntegration" WHERE id = $1"#
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for an asset.
pub async fn resolve_asset_project(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT project_id::uuid FROM "Asset" WHERE id = $1"#
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.into()))
}
```

- [ ] **Step 2: Re-export the functions in `crates/cms-db/src/lib.rs`**
- [ ] **Step 3: Run `cargo check -p cms-db` and verify it passes**

---

### Task 2: Extend `ParentLink` and Centralize Error Masking in `cms-authz`

**Files:**
- Modify: `crates/cms-authz/src/gate.rs`

**Interfaces:**
- Consumes: `cms_db::resolve_*_project`
- Produces:
  - Extended `ParentLink` enum variants: `Theme(Uuid)`, `GitConnection(Uuid)`, `Integration(Uuid)`, `Asset(Uuid)`.
  - `pub async fn require_parent_access(pool: &PgPool, authz: &dyn AuthzBackend, ctx: &AuthzContext, link: ParentLink, relation: Relation) -> Result<Uuid, AppError>`

- [ ] **Step 1: Add new variants to `ParentLink`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentLink {
    Page(Uuid),
    Branch(Uuid),
    Deployment(Uuid),
    ApiKey(Uuid),
    Theme(Uuid),
    GitConnection(Uuid),
    Integration(Uuid),
    Asset(Uuid),
}
```

- [ ] **Step 2: Connect `resolve_parent_project` to `cms-db` functions**

```rust
pub async fn resolve_parent_project(
    pool: &PgPool,
    link: ParentLink,
) -> Result<Option<Uuid>, AppError> {
    match link {
        ParentLink::Page(id) => cms_db::resolve_page_project(pool, id).await,
        ParentLink::Branch(id) => cms_db::resolve_branch_project(pool, id).await,
        ParentLink::Deployment(id) => cms_db::resolve_deployment_project(pool, id).await,
        ParentLink::ApiKey(id) => cms_db::resolve_api_key_project(pool, id).await,
        ParentLink::Theme(id) => cms_db::resolve_theme_project(pool, id).await,
        ParentLink::GitConnection(id) => cms_db::resolve_git_connection_project(pool, id).await,
        ParentLink::Integration(id) => cms_db::resolve_integration_project(pool, id).await,
        ParentLink::Asset(id) => cms_db::resolve_asset_project(pool, id).await,
    }
}
```

- [ ] **Step 3: Ensure `require_parent_access` masks non-existence and denial consistently**

```rust
pub async fn require_parent_access(
    pool: &PgPool,
    authz: &dyn AuthzBackend,
    ctx: &AuthzContext,
    link: ParentLink,
    relation: Relation,
) -> Result<Uuid, AppError> {
    let project_id = resolve_parent_project(pool, link)
        .await?
        .ok_or_else(|| AppError::NotFound("Resource not found".to_string()))?;

    require_project_access(authz, ctx, project_id, relation).await?;

    Ok(project_id)
}
```

- [ ] **Step 4: Update unit tests in `crates/cms-authz/src/gate.rs` and verify `cargo test -p cms-authz`**

---

### Task 3: Implement `ProjectViewer` and `ProjectEditor` Axum Extractors in `cms-auth`

**Files:**
- Create: `crates/cms-auth/src/extractors.rs`
- Modify: `crates/cms-auth/src/lib.rs`

**Interfaces:**
- Produces:
  - `pub struct ProjectViewer { pub auth: AuthUser, pub project_id: Uuid }`
  - `pub struct ProjectEditor { pub auth: AuthUser, pub project_id: Uuid }`
  - Implements `FromRequestParts<S>` for both.

- [ ] **Step 1: Write `crates/cms-auth/src/extractors.rs`**
Implement `FromRequestParts` that:
1. Extracts `Arc<AppState>` from `parts.extensions`.
2. Extracts `AuthUser::from_request_parts(parts, state)`.
3. Parses project ID from URL path (matches `:project_id` or `:id` via path segments / URI parsing).
4. Runs `cms_authz::gate::require_project_access(state.authz_backend(), &auth.authz_context(), project_id, Relation::Viewer / Relation::Editor)`.
5. Returns `ProjectViewer { auth, project_id }` or `ProjectEditor { auth, project_id }`.

- [ ] **Step 2: Export from `crates/cms-auth/src/lib.rs`**
- [ ] **Step 3: Run `cargo check -p cms-auth` and verify compilation**

---

### Task 4: Refactor `cms-themes` to Use Unified Gating

**Files:**
- Modify: `crates/cms-themes/src/handlers.rs`

- [ ] **Step 1: Update direct project routes (`list_themes_handler`, `create_theme_handler`) to use `ProjectViewer` / `ProjectEditor`**
- [ ] **Step 2: Update child routes (`get_theme_handler`, `update_theme_handler`, `delete_theme_handler`) to use `ParentLink::Theme(id)`**
- [ ] **Step 3: Delete `authorize_theme_access` and `authorize_theme_project_access`**
- [ ] **Step 4: Run `cargo test -p cms-themes` and verify passes**

---

### Task 5: Refactor `cms-git` to Use Unified Gating

**Files:**
- Modify: `crates/cms-git/src/handlers.rs`

- [ ] **Step 1: Update `list_connections` and `create_connection` to use `ProjectViewer` / `ProjectEditor`**
- [ ] **Step 2: Update connection-specific child routes (`get_connection`, `update_connection`, `delete_connection`, sync routes) to use `ParentLink::GitConnection(id)`**
- [ ] **Step 3: Delete `authorize_git_connection_access`**
- [ ] **Step 4: Run `cargo test -p cms-git` and verify passes**

---

### Task 6: Refactor `cms-integrations` to Use Unified Gating

**Files:**
- Modify: `crates/cms-integrations/src/handlers.rs`

- [ ] **Step 1: Update `list_integrations` and `create_integration` to use `ProjectViewer` / `ProjectEditor`**
- [ ] **Step 2: Update integration child routes (`get_integration`, `update_integration`, `delete_integration`, `test_integration`) to use `ParentLink::Integration(id)`**
- [ ] **Step 3: Delete `authorize_integration_access`**
- [ ] **Step 4: Run `cargo test -p cms-integrations` and verify passes**

---

### Task 7: Refactor `cms-deployments` to Use Unified Gating

**Files:**
- Modify: `crates/cms-deployments/src/handlers.rs`

- [ ] **Step 1: Update `list_deployments` and `publish_handler` to use `ProjectViewer` / `ProjectEditor`**
- [ ] **Step 2: Update child routes (`get_deployment`, `cancel_deployment`, `retry_deployment`, `logs_deployment`) to use `ParentLink::Deployment(id)`**
- [ ] **Step 3: Delete `authorize_deployment_access`**
- [ ] **Step 4: Run `cargo test -p cms-deployments` and verify passes**

---

### Task 8: Refactor `cms-assets` and `cms-analytics` to Use Unified Gating

**Files:**
- Modify: `crates/cms-assets/src/handlers/asset.rs`
- Modify: `crates/cms-assets/src/handlers/project_asset.rs`
- Modify: `crates/cms-analytics/src/handlers.rs`

- [ ] **Step 1: Update `cms-assets` child routes (`get_asset`, `delete_asset`) to use `ParentLink::Asset(id)`**
- [ ] **Step 2: Update `cms-assets` project routes to use `ProjectViewer` / `ProjectEditor`**
- [ ] **Step 3: Update `cms-analytics` project routes to use `ProjectViewer`**
- [ ] **Step 4: Run `cargo test -p cms-assets -p cms-analytics` and verify passes**

---

### Task 9: Full Workspace Verification and Lints

- [ ] **Step 1: Run full test suite for all modified crates**
  `cargo test -p cms-db -p cms-authz -p cms-auth -p cms-themes -p cms-git -p cms-integrations -p cms-deployments -p cms-assets -p cms-analytics`
- [ ] **Step 2: Run API integration tests**
  `cargo test --test test_17_project_themes --test test_14_deployments --test test_15_git --test test_16_integrations`
- [ ] **Step 3: Run Clippy gate check**
  `cargo clippy --workspace --all-targets -- -D warnings`
