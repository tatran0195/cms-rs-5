# Architecture Design Specification: Modular Monolith Vertical Slices Refactoring

**Date**: 2026-10-12  
**Status**: APPROVED by User ("all in priority order")  
**Reference**: Adapted from `MedBrains` `RFC-SERVER-CRATE-SPLIT.md` for `CMS-RS-4`  

---

## 1. Executive Summary & Objective

In accordance with MedBrains' approved production architecture (`RFC-SERVER-CRATE-SPLIT.md`), CMS is migrating from a traditional horizontal 3-layer architecture (`cms-entity` $\rightarrow$ `cms-db` $\rightarrow$ `cms-biz` $\rightarrow$ `cms-api`) to a **Modular Monolith of Domain Vertical Slices**.

In this architecture:
1. **Zero-Route Server Foundation** (`cms-server-core`) houses shared primitives (`AppState`, middleware, validation, pagination, HTTP tracing). *(Completed)*
2. **Shared Services Layer** (`cms-server-services`) houses cross-domain helpers to break cyclic/sibling dependencies across domain leaves. *(Phase 1)*
3. **Domain Vertical Slices** (`cms-notifications`, `cms-invitations`, `cms-onboarding`, `cms-reports`, `cms-projects`, `cms-documents`) encapsulate their domain end-to-end: models/DTOs, database queries, business services, and Axum HTTP route handlers (`pub fn router(state: Arc<AppState>) -> Router`).

---

## 2. Dependency Hierarchy & Compilation DAG

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                             cms-server (Binary)                             │
│       Mounts domain routers, boots background workers & cron scheduler      │
└──────────────┬────────────────────────┬─────────────────────────────┬───────┘
               │                        │                             │
               ▼                        ▼                             ▼
┌──────────────────────────┐ ┌──────────────────────────┐ ┌───────────────────┐
│     cms-invitations      │ │      cms-onboarding      │ │    cms-reports    │
│  (Domain Vertical Slice) │ │  (Domain Vertical Slice) │ │  (Vertical Slice) │
└──────────────┬───────────┘ └──────────┬───────────────┘ └───────────┬───────┘
               │                        │                             │
               └───────────────────┬────┴─────────────────────────────┘
                                   │
                                   ▼
               ┌───────────────────────────────────────┐
               │          cms-server-services          │
               │   Shared cross-domain logic & sinks   │
               └───────────────────┬───────────────────┘
                                   │
                                   ▼
               ┌───────────────────────────────────────┐
               │           cms-server-core             │
               │  AppState, Middleware, Primitives     │
               └───────┬───────────────────────┬───────┘
                       │                       │
                       ▼                       ▼
               ┌───────────────┐       ┌───────────────┐
               │    cms-db     │       │  cms-outbox   │
               └───────────────┘       └───────────────┘
```

### Dependency Rules:
1. **Vertical slices are true leaves**: A domain leaf crate (e.g. `cms-invitations`) MUST NEVER depend on a sibling domain leaf crate (e.g. `cms-onboarding`).
2. **Cross-domain orchestration**: If two domains need shared business logic or coordinate mutations, the helper logic belongs in `cms-server-services`, not inside either leaf crate.
3. **Pure Move / Zero Regression**: Behavior and API route contracts remain 100% backwards-compatible. `cms-api` delegates routes to the vertical slice crates.

---

## 3. Implementation Phases & Priority Order

### Phase 1: Shared Services Layer (`crates/cms-server-services`)
**Purpose**: Mirror MedBrains Step 3 (`medbrains-server-services`) to break cross-domain dependencies and provide shared integration helpers.
- **Crate**: `crates/cms-server-services`
- **Dependencies**: `cms-db`, `cms-entity`, `cms-error`, `cms-server-core`, `cms-outbox`, `cms-queue`, `cms-storage`.
- **Modules**:
  - `auth_audit`: Cross-domain audit logging helpers (recording security events, HTTP audit entry dispatch).
  - `storage_url`: Presigned URL generation, asset resolution, export artifact URLs.
  - `tenant_context`: Shared workspace resolution, permissions verification helper.
  - `event_dispatcher`: Cross-domain event publication (outbox and realtime fan-out helpers).

### Phase 2: Platform Subsystem Vertical Slices
Extract the existing subsystems (which already have validated database tables and business logic) into dedicated, self-contained vertical slice crates:

#### 2.1 `crates/cms-invitations`
- **Consolidates**:
  - DB queries: `crates/cms-db/src/invitation.rs`
  - Business service: `crates/cms-biz/src/invitation.rs`
  - Axum handlers: `crates/cms-api/src/invitation/`
- **Router**:
  - `GET /api/app/invitations`
  - `POST /api/app/invitations`
  - `DELETE /api/app/invitations/{id}`
  - `GET /api/app/invitations/accept`
  - `POST /api/app/invitations/accept`
- **Tests**: `crates/cms-invitations/tests/invitations_test.rs`

#### 2.2 `crates/cms-onboarding`
- **Consolidates**:
  - DB queries: `crates/cms-db/src/onboarding.rs`
  - Business service: `crates/cms-biz/src/onboarding.rs`
  - Axum handlers: `crates/cms-api/src/setup/`
- **Router**:
  - `GET /api/setup/status`
  - `POST /api/setup/complete`
  - `GET /api/auth/verify-email`
  - `POST /api/auth/resend-verification`
- **Tests**: `crates/cms-onboarding/tests/onboarding_test.rs`

#### 2.3 `crates/cms-reports`
- **Consolidates**:
  - DB queries: `crates/cms-db/src/report.rs`
  - Business service: `crates/cms-biz/src/report.rs`
  - Axum handlers: `crates/cms-api/src/export/` (report runs endpoints)
- **Router**:
  - `GET /api/app/reports`
  - `POST /api/app/reports`
  - `GET /api/app/reports/{id}`
  - `GET /api/app/reports/{id}/download`
  - `DELETE /api/app/reports/{id}`
- **Tests**: `crates/cms-reports/tests/reports_test.rs`

### Phase 3: Core Documentation Domain Vertical Slices
1. **`crates/cms-projects`**:
   - Workspace association, project metadata, branches, themes, project member roles.
2. **`crates/cms-documents`**:
   - Document pages, hierarchical trees, markdown rendering, revisions, inline comments.

---

## 4. Verification & Non-Negotiable Constraints

1. **Rust Edition 2024 (MSRV 1.85)** with `forbid(unsafe_code)`.
2. **Strict Clippy Compliance**: Zero warnings on `cargo clippy --all-targets --all-features`.
3. **No Placeholders**: Every file, function, query, handler, and test is 100% complete and working.
4. **Internal Platform Scope**: Zero billing, zero payment gateways, zero marketing pages (ADR 0001).
5. **Windows Build Safety**: Compile with `-j 2` to avoid stack buffer overruns.
