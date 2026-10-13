# Architecture Design Specification: Consolidating Infrastructure Crates into `cms-infra` & Crate Cleanup

- **Date**: 2026-10-13
- **Status**: Approved by User
- **Topic**: Crate Consolidation and Cleanup: Unifying Infrastructure Crates into `cms-infra`
- **Related ADRs**: [ADR 0007: Modular Monolith Transition to Vertical Slices](../../adr/0007-vertical-slices-modular-monolith.md)

---

## 1. Executive Summary & Goals

The CMS codebase currently contains 41 distinct crates. While business logic and HTTP endpoints have been migrated into high-cohesion domain vertical slices (ADR 0007), infrastructure concerns remain fragmented across 7 individual micro-crates (`cms-config`, `cms-storage`, `cms-search`, `cms-queue`, `cms-mailer`, `cms-analytics`, `cms-i18n`), alongside an obsolete facade crate (`cms-middleware`).

This specification defines the consolidation of these 7 infrastructure crates into a unified, high-cohesion `crates/cms-infra` crate, the elimination of the `cms-middleware` facade in favor of direct `cms-server-core` usage, and the complete cleanup of legacy crate artifacts.

### Key Objectives
1. **Unified Infrastructure Layer**: Consolidate `cms-config`, `cms-storage`, `cms-search`, `cms-queue`, `cms-mailer`, `cms-analytics`, and `cms-i18n` into submodules under a single `crates/cms-infra` crate.
2. **Preserve Fine-Grained Features**: Maintain optional feature gates (`s3`, `vector`, `memory`) in `cms-infra` so heavy external dependencies (AWS SDK, FastEmbed) remain opt-in.
3. **Eliminate Facades and Shims**: Remove `crates/cms-middleware` completely; update all consumers to use `cms-server-core` directly.
4. **Purge Obsolete Directories**: Remove old directories for the 7 merged crates, and finalize removal of deleted `crates/cms-api` and `crates/cms-biz` files from git tracking.
5. **Zero Functional Regression**: All existing behaviors, public interfaces, and API contracts remain strictly preserved and compile cleanly.

---

## 2. Target Architecture & Module Layout

### 2.1 Crate Layout: `crates/cms-infra`

```
crates/cms-infra/
├── Cargo.toml
├── build.rs                          # Bundles Minijinja email templates and i18n JSON catalogs
└── src/
    ├── lib.rs                        # Public re-exports: config, storage, search, queue, mailer, analytics, i18n
    ├── config/                       # Configuration schemas & Figment loader (ex-cms-config)
    │   ├── mod.rs
    │   ├── analytics.rs
    │   ├── auth.rs
    │   ├── database.rs
    │   ├── mailer.rs
    │   ├── mcp.rs
    │   ├── queue.rs
    │   ├── search.rs
    │   ├── security.rs
    │   ├── server.rs
    │   ├── site.rs
    │   └── storage.rs
    ├── storage/                      # Pluggable Local & S3 object storage (ex-cms-storage)
    │   ├── mod.rs
    │   ├── traits.rs
    │   ├── local.rs
    │   └── s3.rs
    ├── search/                       # Hybrid Tantivy + Lindera + FastEmbed search (ex-cms-search)
    │   ├── mod.rs
    │   ├── tantivy_engine.rs
    │   ├── lindera_tokenizer.rs
    │   ├── vector.rs
    │   ├── plain_text.rs
    │   └── traits.rs
    ├── queue/                        # Task queue memory/db abstractions & worker reaper (ex-cms-queue)
    │   ├── mod.rs
    │   ├── traits.rs
    │   ├── memory.rs
    │   ├── db.rs
    │   └── reaper.rs
    ├── mailer/                       # Lettre + MiniJinja transactional emailer (ex-cms-mailer)
    │   ├── mod.rs
    │   ├── traits.rs
    │   ├── service.rs
    │   └── renderer.rs
    ├── analytics/                    # Operational metrics & telemetry engine (ex-cms-analytics)
    │   ├── mod.rs
    │   └── service.rs
    └── i18n/                         # BCP-47 locale negotiation & translation catalogs (ex-cms-i18n)
        ├── mod.rs
        ├── catalog.rs
        └── negotiator.rs
```

### 2.2 Public Namespace

Consumers import `cms-infra = { workspace = true }` and access services via clear namespaces:
* `cms_infra::config::{Config, DatabaseConfig, StorageConfig, ...}`
* `cms_infra::storage::{Storage, LocalStorage, S3Storage, create_storage}`
* `cms_infra::search::{SearchEngine, TantivySearchEngine, create_search_engine}`
* `cms_infra::queue::{JobQueue, JobEnvelope, JobType, create_job_queue_with_pool}`
* `cms_infra::mailer::{Mailer, EmailRenderer, create_mailer}`
* `cms_infra::analytics::{AnalyticsService, MetricEvent}`
* `cms_infra::i18n::{Locale, Catalog, translate, negotiate_locale}`

---

## 3. Dependency Hierarchy & Consumers

### 3.1 Crate Dependency DAG

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                             cms-server (Binary)                             │
│       Mounts domain routers, boots background workers & cron scheduler      │
└──────────────┬────────────────────────┬─────────────────────────────┬───────┘
               │                        │                             │
               ▼                        ▼                             ▼
┌──────────────────────────┐ ┌──────────────────────────┐ ┌───────────────────┐
│     Vertical Slices      │ │     cms-server-core      │ │    cms-worker     │
│ (reports, content, etc.) │ │ (AppState, Middleware)   │ │ (Worker Process)  │
└──────────────┬───────────┘ └──────────┬───────────────┘ └───────────┬───────┘
               │                        │                             │
               └───────────────────┬────┴─────────────────────────────┘
                                   │
                                   ▼
                       ┌───────────────────────┐
                       │       cms-infra       │
                       │ config, storage, etc. │
                       └───────────┬───────────┘
                                   │
                     ┌─────────────┼─────────────┐
                     ▼             ▼             ▼
               ┌───────────┐ ┌───────────┐ ┌───────────┐
               │ cms-entity│ │  cms-db   │ │ cms-error │
               └───────────┘ └───────────┘ └───────────┘
```

### 3.2 Migrating Consumers

1. **`crates/cms-server-core`**:
   - Replaces dependencies on `cms-config`, `cms-storage`, `cms-queue`, `cms-search`, `cms-analytics`, `cms-mailer`, `cms-i18n` with `cms-infra = { path = "../cms-infra" }`.
   - In `app_state.rs`: imports change from `cms_config::Config` to `cms_infra::config::Config`, `cms_storage::Storage` to `cms_infra::storage::Storage`, etc.
   - In middleware (e.g. `locale.rs`): imports change to `cms_infra::i18n::Locale`.
2. **`crates/cms-server-services`**:
   - Uses `cms-infra` for `storage` (presigned URLs) and `queue` (event dispatcher).
3. **`crates/cms-worker`**:
   - Uses `cms-infra` for job queue, search indexing, storage downloads, and mailer delivery.
   - Replaces `cms-middleware` dependency with direct `cms-server-core` dependency.
4. **`crates/cms-sites` & `crates/cms-notifications`**:
   - Replaces `cms-middleware` with `cms-server-core`.
5. **Vertical Slices (`cms-reports`, `cms-assets`, `cms-deployments`, `cms-git`)**:
   - Replace direct dependencies on `cms-storage`, `cms-queue`, `cms-mailer` with `cms-infra`.
6. **`apps/api`**:
   - Replaces `cms-config`, `cms-queue`, `cms-worker`, `cms-search`, `cms-analytics`, `cms-middleware` with `cms-infra` and `cms-server-core`.

---

## 4. Deletion & Cleanup Strategy

1. **Delete obsolete crates**:
   - `crates/cms-config`
   - `crates/cms-storage`
   - `crates/cms-search`
   - `crates/cms-queue`
   - `crates/cms-mailer`
   - `crates/cms-analytics`
   - `crates/cms-i18n`
   - `crates/cms-middleware`
2. **Git status cleanup**:
   - Stage deletion of previously removed `crates/cms-biz` and `crates/cms-api` files.
   - Remove workspace members from root `Cargo.toml`.
3. **Documentation**:
   - Update `Cargo.toml` `[workspace.dependencies]` and `members`.
   - Update `AGENTS.md` and `README.md` to reflect `cms-infra`.

---

## 5. Non-Negotiable Constraints & Verification

1. **Rust Edition 2024 (MSRV 1.85)** with `forbid(unsafe_code)`.
2. **Strict Clippy Compliance**: Zero warnings on `cargo clippy --workspace --all-targets`.
3. **No Placeholders**: 100% working code across all submodules.
4. **Windows Safe Build**: Respect file locking and build stability.
5. **Verification Suite**:
   - `cargo check --workspace` passes.
   - `cargo test -p cms-infra` passes.
   - `cargo check -p cms-server` passes.
