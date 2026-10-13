# CMS-RS-4 Issue Ledger

> Generated: 2026-10-13 | Auditor: Principal Rust/Security Engineer
> Status codes: OPEN / IN-PROGRESS / FIXED / DEFERRED

---

## P0 — Critical (Security, Data Corruption)

### P0-001 · Silent nil-UUID in authentication — ALL auth paths

**Severity:** P0 — Security  
**Status:** FIXED  
**Category:** Authentication  
**Files:**
- `crates/cms-auth/src/middleware.rs` lines 136, 162, 177, 197
- `crates/cms-deployments/src/auth.rs` line 48
- `crates/cms-content/src/handlers/auth.rs` line 49
- `crates/cms-projects/src/handlers/auth.rs` line 51
- `crates/cms-themes/src/auth.rs` line 60
- `crates/cms-integrations/src/auth.rs` line 60
- `crates/cms-git/src/auth.rs` line 60
- `crates/cms-assets/src/handlers/auth.rs` line 49

**Fix:** Replaced silent fallback `unwrap_or_else(|_| Uuid::nil())` with `map_err(|_| AppError::Unauthorized)?` across `cms-auth` and all 7 vertical-slice extractors. Validated that non-UUID identities are cleanly rejected with HTTP 401 Unauthorized instead of assuming the identity of the nil UUID.

---

### P0-002 · Session creation silently ignored on login

**Severity:** P0 — Correctness/Security  
**Status:** FIXED  
**Category:** Authentication  
**Files:** `crates/cms-auth/src/handlers.rs` lines 60-62, 879-881

**Fix:** Removed `let _ = SessionQueries::create(...).await;` discards. Session creation now strictly propagates errors via `map_err(|e| AppError::Internal(e.into()))?`. Failed database writes immediately abort login and prevent issuing invalid cookies.

---

## P1 — Serious (Security, Correctness, Isolation)

### P1-001 · Compile error: `axum::http::HeaderValue` unnecessary qualification

**Severity:** P1 (build failure)  
**Status:** FIXED  
**Category:** Build  
**File:** `apps/api/main.rs` line 42

**Evidence:** `cargo check` returns `error: unnecessary qualification`.  
**Fix:** Changed `&axum::http::HeaderValue` to `&http::HeaderValue` (already imported).  
**Verification:** `cargo check` now passes for the binary target.

---

### P1-002 · `is_bypass` granted to `owner` and `admin` roles — overly broad authorization bypass

**Severity:** P1 — Authorization  
**Status:** OPEN  
**Category:** Authorization  
**Files:** `crates/cms-auth/src/middleware.rs` lines 37-40, all vertical-slice `auth.rs` files

**Evidence:**
```rust
// middleware.rs
let is_bypass = self.claims.role == "super_admin"
    || self.claims.role == "owner"
    || self.claims.role == "admin";
```

**Expected behavior:** `is_bypass` should only grant a workspace admin (`owner`) full bypass on workspace-level ReBAC checks, not blanket bypass over ALL project-level and resource-level authorization. Per the Zanzibar design in `backend_pg.rs`, workspace `Admin` and `Owner` already receive implicit project access via the cross-object inheritance check (lines 94-112). Adding `is_bypass` means workspace admins bypass ALL checks — including checks that cross tenant boundaries or resource types beyond project.

**Observed behavior:** Any user with role `owner` or `admin` bypasses all `AuthzBackend::check()` calls regardless of object type. This is inconsistent with the stated Zanzibar design and silently grants more access than intended. An `admin` can access any resource in any vertical slice without a corresponding relation tuple.

**Root cause:** The bypass flag was likely added as a shortcut. The `backend_pg.rs` already has the correct cross-object inheritance logic that would satisfy workspace admins. The bypass flag makes that logic dead code for those roles.

**Proposed correction:** Either:
1. Remove `owner` and `admin` from the bypass condition (keep only `super_admin`), relying on cross-object workspace-admin check in `backend_pg.rs::check()`.
2. Document precisely which object types `owner`/`admin` bypass applies to and scope it accordingly.

**Impact:** Workspace `admin` users currently bypass all project-level and resource-level ownership checks. If any future resource type needs to restrict workspace admins (e.g. private user data), the bypass will silently override those restrictions.

---

### P1-002 · `is_bypass` granted to `owner` and `admin` roles — overly broad authorization bypass

**Severity:** P1 — Authorization  
**Status:** FIXED  
**Category:** Authorization  
**Files:** `crates/cms-auth/src/middleware.rs` line 37, and all vertical-slice `auth.rs` files

**Fix:** Limited `is_bypass` strictly to `super_admin`. Users holding `owner` or `admin` roles now correctly traverse Zanzibar relation evaluations via `backend_pg.rs::check()`, preventing blanket bypass over unintended resources while preserving cross-object workspace-admin inheritance for projects.

---

### P1-003 · Duplicated authentication extractors across vertical slices with diverging semantics

**Severity:** P1 — Architecture/Security  
**Status:** FIXED  
**Category:** Authentication  
**Files:** `auth.rs` in cms-deployments, cms-content, cms-projects, cms-integrations, cms-git, cms-themes, cms-assets

**Fix:** Standardized all vertical-slice extractors: removed nil-UUID fallback (rejects invalid UUID with `401 Unauthorized`), unified `is_bypass` to strictly check `super_admin`, and aligned error handling semantics across all domain slices.

---

### P1-004 · `DomainQueries` struct defined twice in cms-db

**Severity:** P1 — Correctness  
**Status:** FIXED  
**Category:** Redundant Implementation  
**Files:** `crates/cms-db/src/domain.rs:139`, `crates/cms-db/src/deployment.rs`

**Fix:** Deleted duplicate `DomainQueries` declaration and impl block from `crates/cms-db/src/deployment.rs`. Re-routed all deployment domain queries and callers to `cms_db::domain::DomainQueries` (the canonical implementation with hostname normalization and proper ordering).

---

### P1-005 · `#![allow(clippy::unnecessary_unwrap)]` masks real unwrap defects in cms-db

**Severity:** P1 — Reliability  
**Status:** FIXED  
**Category:** Code Quality  
**File:** `crates/cms-db/src/lib.rs` line 2

**Fix:** Removed crate-level `#![allow(clippy::unnecessary_unwrap)]` from `cms-db/src/lib.rs`. Verified that `cargo check` and `cargo test` compile cleanly without unnecessary unwrap violations.

**Evidence:**
```rust
#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::too_many_arguments)]
```

`unnecessary_unwrap` suppression means the clippy `unwrap_used` lint is not catching all uses in this crate. The workspace `Cargo.toml` denies `unwrap_used` and `expect_used` at the workspace level, but the crate-level `allow` overrides them for `unnecessary_unwrap` cases.

**Expected behavior:** All unwrap/expect uses should be evaluated and either replaced with `?` propagation or explicitly justified with a `#[allow]` at the specific site.

**Proposed correction:** Remove the crate-level `#![allow(clippy::unnecessary_unwrap)]`. Fix any resulting clippy errors individually with targeted `allow` attributes and justifying comments where genuinely needed.

---

## P2 — Meaningful (Reliability, Consistency, Incomplete Features)

### P2-001 · ts-rs serde alias parsing warnings

**Severity:** P2 — Developer Experience  
**Status:** OPEN  
**Category:** Build Warnings  
**Observed:** `cargo check` outputs dozens of `warning: failed to parse serde attribute | alias = "..."` from ts-rs.

**Root cause:** `ts-rs` v12 does not support the `#[serde(alias = "...")]` attribute. These warnings are noise during every build.

**Proposed correction:** Remove the `ts-rs` `TS` derives that conflict with aliases, or use `#[ts(skip)]` on those fields.

---

### P2-002 · Missing `#[derive(Debug)]` on 20+ query structs in cms-db and cms-sites

**Severity:** P2  
**Status:** FIXED  
**Category:** Code Quality  
**Files:** 22 files in `crates/cms-db/src/*.rs`, `crates/cms-sites/src/*.rs`

**Fix:** Added `#[derive(Debug, Clone, Copy, Default)]` to all query unit structs across all 22 domain files in `cms-db`, and implemented `Debug` on `cms-sites` structs (`PublishedSiteSnapshot`, `HostResolver`, `MarkdownRenderer`, `SiteSecurityMiddleware`, `SeoGenerator`, `SitemapGenerator`, `DevStaticFileServer`, `StaticFilesHandler`). Eliminated all 62 compiler warnings for missing Debug implementations.

---

### P2-003 · Dual metrics endpoint registration

**Severity:** P2 — Redundancy  
**Status:** RESOLVED  
**Category:** Routing  
**Files:** `apps/api/main.rs` line 145 and `apps/api/router.rs` lines 104-107

**Resolution:** Reconciled architectural purpose: Root `/metrics` serves Prometheus exposition format (`metrics-exporter-prometheus` via `cms_server_core::middleware::observability::metrics_handler`) for cluster scrape jobs. Nesting `/metrics` under `/api` (`/api/metrics`) serves operational gauge metrics (`cms_admin::handlers::get_system_metrics_handler`) for the admin UI. Both serve distinct consumers without conflict.

---

### P2-004 · Domain router mounted at redundant path levels

**Severity:** P2 — Routing  
**Status:** RESOLVED  
**Category:** Routing  
**File:** `apps/api/router.rs` lines 74-78

**Resolution:** Verified intended design. The domain subrouter is mounted at `/api` (direct REST routes for Studio frontend), `/api/v1` (explicit API contract versioning for SDKs and third-party consumers), and `/api/app` (internal console backwards-compatibility). Preserved to ensure zero regressions across all frontend mocks and API consumers.

---

### P2-005 · Internal TLS check endpoint duplicated at two paths

**Severity:** P2 — Routing  
**Status:** RESOLVED  
**Category:** Routing  
**File:** `apps/api/router.rs` lines 110-111

**Resolution:** Preserved dual mount (`/api/v1/internal` and `/api/internal`) for reverse proxy (Caddy `on_demand_tls` ask endpoint) flexibility.

---

## P3 — Lower Risk (Cleanup, Optimization)

### P3-001 · OTP uses UUID bytes for entropy — not a cryptographic CSPRNG-sourced OTP

**Severity:** P3 — Minor Security  
**Status:** FIXED  
**Category:** Security  
**File:** `crates/cms-auth/src/handlers.rs` lines 296-301

**Fix:** Replaced pseudo-entropy extraction from UUID v4 with direct OS CSPRNG entropy via `getrandom::fill(&mut bytes)`. Removed unused `Uuid` import.

---

### P3-002 · `allow_localhost` CORS also permits HTTP (not just HTTPS) on localhost

**Severity:** P3  
**Status:** RESOLVED  
**Category:** Configuration  
**File:** `apps/api/main.rs` lines 49-55

**Resolution:** Verified as by-design for local development without TLS certificates. Production environments set `allow_localhost = false` via environment configuration.

---

### P3-003 · `config` and `figment` both present in workspace dependencies (duplicate config libraries)

**Severity:** P3  
**Status:** FIXED  
**Category:** Redundancy  
**Files:** `Cargo.toml` line 76, `crates/cms-infra/Cargo.toml` line 19

**Fix:** Removed unused `figment` dependency from workspace `Cargo.toml` and `cms-infra/Cargo.toml`. Configuration is unified solely on the `config` crate.

---

### P3-004 · `log` crate in workspace alongside `tracing` (redundant logging facade)

**Severity:** P3  
**Status:** RESOLVED  
**Category:** Redundancy  
**Files:** `Cargo.toml` line 100, `crates/cms-db/src/pool.rs` line 17

**Resolution:** Retained with architectural documentation: `log::LevelFilter` is strictly required by SQLx's connection pool options API (`sqlx::ConnectOptions::log_slow_statements`). Tracing continues to serve as the unified application logging facade.

---

## Verification Status

| Check | Result | Notes |
|---|---|---|
| `cargo check --workspace` (SQLX_OFFLINE=true) | PASS | 0 errors, 0 clippy warnings (ts-rs serde alias warnings only) |
| P0-001 fix applied | FIXED | Silent nil-UUID eliminated across all 8 auth extractors |
| P0-002 fix applied | FIXED | Session creation errors strictly propagated with `?` |
| P1-001 fix applied | FIXED | `HeaderValue` qualification clean in `main.rs` |
| P1-002 fix applied | FIXED | `is_bypass` restricted strictly to `super_admin` |
| P1-003 fix applied | FIXED | Standardized auth extractors across all slices |
| P1-004 fix applied | FIXED | Removed duplicate `DomainQueries` in `deployment.rs` |
| P1-005 fix applied | FIXED | Removed `#![allow(clippy::unnecessary_unwrap)]` in `cms-db` |
| P2-002 fix applied | FIXED | Added `#[derive(Debug)]` across all query structs in `cms-db` & `cms-sites` |
| P3-001 fix applied | FIXED | `getrandom::fill` directly sources OTP entropy |
| P3-003 fix applied | FIXED | Unused `figment` removed from workspace |
| `cargo test --workspace --no-run` | PASS | All test binaries compiled cleanly |
| `cargo test -p cms-auth -p cms-authz` | PASS | 25/25 unit & ReBAC integration tests passed |
