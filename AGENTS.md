# AGENTS.md — CMS Modern Documentation Platform

## Project Overview

CMS is a modern, fast, searchable documentation platform built with Rust and React for **internal company use**. It provides a notion-style markdown editor, project workspace management, multi-lingual search (with native Japanese support), and high-performance reader sites for company technical documentation.

---

## 🚫 Critical Non-Negotiable Constraints (MANDATORY)

These constraints are permanent and binding. **Never violate or relax them:**

### 1. No Plan, Billing, or Pricing Features
**DO NOT attempt to re-introduce, recover, scaffold, or re-implement any Plan, Billing, or Subscription features:**
- **No Plan / Billing Sections**: Do **NOT** add `PlanSection`, `plan` tabs, or billing management to project settings (`/app/projects/$projectId/settings`), workspace settings (`/app/settings`), or dashboard overviews.
- **No Plan Tiers**: Do **NOT** add pricing tiers or subscription models (such as Free, Pro, Team, Enterprise, Self-Hosted vs Cloud pricing).
- **No Payment Gateways**: Do **NOT** integrate Stripe, Polar, LemonSqueezy, Paddle, or any payment gateway.
- **No Billing Quota Enforcement**: Do **NOT** block user features, restrict workspaces, or show upgrade/downgrade prompts based on billing plans or payment statuses.
- **Diagnostics vs Billing**: Diagnostic usage tracking (such as page counts or storage metrics) is strictly internal operational telemetry. It must never be tied to billing cycles, invoices, payment statuses, or plan tiers.

### 2. No Marketing, Landing, or Blog Pages
**DO NOT re-create marketing pages, promotional blogs, comparison roundups, or public landing sites:**
- **Homepage**: The root route `/` intentionally redirects directly to `/app`. The `/app` route serves as the primary application homepage for authenticated company team members.
- **No Marketing Landing Pages**: Do **NOT** replace `/` with a promotional or marketing landing page.
- **No Marketing Routes**: Do **NOT** add `/pricing`, `/cloud`, `/compare/*`, `/alternatives/*`, `/blog/*`, `/about`, `/contact`, `/developers`, `/guides` marketing hubs, or marketing SEO schemas.
- **Authentication**: Auth routes (`/sign-in`, `/sign-up`) are for internal team members and must not require public SaaS terms of service or privacy policy checkboxes.

---

## Canonical Tech Stack

### Backend
| Layer | Technology |
|---|---|
| Language | Rust (edition 2024, MSRV 1.85) |
| Web framework | Axum 0.8 + Tower 0.5 |
| Async runtime | Tokio (full features) |
| Serialization | serde + serde_json |
| Error handling | `thiserror` (library crates), `cms_error::AppError` (application edges) |
| Logging & Metrics | `tracing` + `tracing-subscriber` (structured, JSON), `metrics` |
| Auth | In-process JWT (`jsonwebtoken`), sessions, Argon2 password hashing |

### Databases & Persistence
| Component | Technology | Purpose |
|---|---|---|
| PostgreSQL 14+ | Primary Relational Store | Users, workspaces, projects, pages, revisions, assets, comments |
| SQLx | Compile-Time Checked SQL | `sqlx::query!`, `sqlx::query_as!`, `sqlx::query_scalar!` |
| Migrations | `sqlx::migrate!()` | Embedded at compile time in `cms-db` |

### Search & Infrastructure
| Capability | Technology | Purpose |
|---|---|---|
| Full-text Search | Tantivy (0.26) | BM25 inverted index stored on disk |
| Japanese Tokenizer | Lindera (6.0) with SudachiDict | In-process Japanese morphological analysis |
| Semantic Vector Search | FastEmbed (7.1) | In-process ONNX embeddings (`multilingual-e5-small`) |
| Object Storage | Pluggable (Local / S3) | Local disk (`C:/cms/uploads`) or AWS S3 |
| Asynchronous Queue | In-memory / Redis | Background jobs via `cms-queue` & `cms-worker` |
| Email Delivery | Lettre + MiniJinja | Transactional notifications and invitations |
| Agent Integration | rmcp (3.5) | Model Context Protocol server exposing CMS capabilities |

### Web Frontend
| Layer | Technology | Purpose |
|---|---|---|
| Framework | React 18 + TypeScript (strict) | Studio console (`apps/studio`) and Reader (`apps/site`) |
| Bundler | Vite | Fast HMR and optimized production bundling |
| Routing | React Router v7 | Nested data routing and loaders |
| Server state | TanStack Query v5 | Remote server data cache, refetching, invalidation |
| Client state | Zustand | Durable client state (auth user, active project, editor layout) |
| Form state | React Hook Form + Zod | Schema-validated form submissions via `zodResolver` |
| Linter / Formatter | Biome | Strict linting and formatting (`biome check`) |

---

## Project Structure & Architecture

The backend follows a strict **3-Layer Architecture** complemented by specialized infrastructure crates:

```
cms-rs-4/
├── Cargo.toml                    # Rust workspace root
├── rust-toolchain.toml           # Rust 1.85 toolchain
├── rustfmt.toml                  # Code formatting rules
├── deny.toml                     # Cargo dependency verification
├── config/                       # Configuration profiles (dev.env, deploy.toml)
├── docs/                         # Documentation & ADRs
│   └── adr/                      # Architecture Decision Records
├── crates/
│   ├── cms-entity/               # Layer 0: Pure domain types, models, DTOs (Zero deps)
│   ├── cms-db/                   # Layer 1: PostgreSQL repositories, queries, migrations
│   ├── cms-biz/                  # Layer 2: Business logic services & domain orchestration
│   ├── cms-api/                  # Layer 3: Axum HTTP routers, extractors, and handlers
│   ├── cms-server/               # Composition root binary (starts HTTP server & workers)
│   │
│   ├── cms-auth/                 # Session, JWT, password, cookie security
│   ├── cms-authz/                # Role & permission checks
│   ├── cms-authz-gate/           # Middleware authorization gates
│   ├── cms-api-keys/             # API key lifecycle and validation
│   ├── cms-mfa/                  # Multi-factor authentication
│   ├── cms-oauth/                # OAuth identity providers
│   ├── cms-storage/              # Pluggable storage (Local / S3)
│   ├── cms-search/               # Tantivy + Lindera + FastEmbed search engine
│   ├── cms-queue/                # Task queue abstraction (Memory / Redis)
│   ├── cms-worker/               # Background task processor
│   ├── cms-mailer/               # Transactional email service
│   ├── cms-mcp/                  # MCP server endpoints for AI tools
│   ├── cms-i18n/                 # Localization & internationalization
│   ├── cms-analytics/            # Internal operational metrics & usage telemetry
│   ├── cms-error/                # AppError and HTTP error responses
│   ├── cms-config/               # Figment configuration loader
│   ├── cms-middleware/           # Observability, rate limit, CORS, security headers
│   └── cms-sites/                # Static documentation site generator & SPA server
└── apps/
    ├── studio/                   # Notion-style documentation management frontend
    └── site/                     # Published documentation reader site
```

### Layer Dependency Rules:
1. `cms-entity` has **zero** internal crate dependencies.
2. `cms-db` depends only on `cms-entity` and database drivers. It **never** depends on `cms-biz` or `cms-api`.
3. `cms-biz` depends on `cms-entity`, `cms-db`, and infrastructure crates. It **never** depends on `cms-api` or HTTP concepts.
4. `cms-api` depends on `cms-biz`, `cms-entity`, `cms-db`. It handles HTTP routing and status codes.
5. `cms-server` is the composition root binary assembling all crates.

---

## Coding Standards

### Rust

- **Edition 2024**, minimum Rust version **1.85**.
- `unsafe` code is **strictly forbidden** (`forbid(unsafe_code)`).
- **Strict Clippy**: `clippy::all` denied, `clippy::pedantic` + `clippy::nursery` warned.
- The following are **denied** (will fail compilation):
  - `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`
  - `dbg_macro`, `print_stdout`, `print_stderr`
  - `wildcard_imports`, `clone_on_ref_ptr`, `inefficient_to_string`
- Use the `?` operator for error propagation. Return `Result<T, AppError>` from fallible functions.
- Use `thiserror` for library error definitions; use `cms_error::AppError` at HTTP boundaries.
- Format with `cargo fmt`.

### SQL & Database

- **All queries must be compile-time verified** via `sqlx::query!`, `sqlx::query_as!`, or `sqlx::query_scalar!`.
- `.sqlx/` offline metadata is committed; normal build/check/test runs with `SQLX_OFFLINE=true`.
- Run `cargo sqlx prepare` only against migrated local/staging databases, never production.
- Every workspace-scoped query must enforce `workspace_id = $1`.

### `cms-db` Public Surface

Every write handler MUST call `cms_db::set_write_context(&mut tx, workspace_id, user_id, ip)` 
immediately after `pool.begin()`. This sets RLS context and audit attribution in one round-trip.

For read-only queries without a transaction, use `cms_db::workspace_conn(pool, workspace_id)`.

Key public APIs:
- `cms_db::create_pool(url)` / `cms_db::create_pool_with_config(url, &config)`
- `cms_db::health_check(pool)`
- `cms_db::set_write_context(&mut tx, workspace_id, user_id, ip)`
- `cms_db::set_read_context(&mut tx, workspace_id)`
- `cms_db::set_extended_audit_context(&mut tx, ...)`
- `cms_db::workspace_conn(pool, workspace_id)`
- `cms_db::next_sequence(&mut tx, workspace_id, seq_type, prefix, pad_width)`
- `cms_db::AuditLogger::log(&mut tx, &entry)`
- `cms_db::AuditLogger::log_http(pool, &http_entry)`
- `cms_db::AuditLogger::verify_chain_for_workspace(pool, workspace_id)`

### TypeScript & React

- **Strict TypeScript** (`strict: true`).
- **Biome** for linting and formatting (`biome check`, `biome format`).
- **Form State Rules**:
  - **All user-editable form payloads must use React Hook Form** (`useForm`, `Controller`, `FormProvider`). Prohibit one `useState` per field.
  - Form validation must use **Zod** (`zodResolver(schema)`). Schemas must match backend Rust request models.
  - Do NOT synchronize form fields via `useEffect`. Initialize with `defaultValues`, call `reset()` on record change, derive payloads in `handleSubmit`.
  - Modal and drawer visibility: pair toggle with form `reset()`.
  - Avoid TypeScript `as` type assertions. Align form types directly with domain models.
- **Data Access Rules**:
  - All API calls through TanStack Query (`useQuery`, `useMutation`) via typed service hooks.
  - No raw `fetch` inside UI components.
  - Zustand is reserved for durable client state only (session, active workspace/project, editor UI state). Never store server query caches in Zustand.
- **useEffect Policy**:
  - **Avoid `useEffect` unless absolutely necessary.**
  - Never use `useEffect` for data fetching or derived state.
  - Allowed only for external browser/DOM listeners (keyboard shortcuts, resize, WebSocket events).
- **Dead Code Rule**:
  - Delete unused code immediately. No unused imports, variables, types, or commented-out blocks.

---

## Architecture Decision Records (ADRs)

Foundational decisions are documented in `docs/adr/`:

| ADR | Title | Key Decision |
|---|---|---|
| [ADR 0001](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0001-internal-company-scope.md) | Internal Scope & No Billing/Marketing | Permanent prohibition of SaaS billing and public marketing routes |
| [ADR 0002](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0002-three-layer-crate-architecture.md) | Three-Layer Crate Architecture | Entity -> DB -> Biz -> API layering with dedicated infrastructure |
| [ADR 0003](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0003-sqlx-compile-time-no-orm.md) | SQLx Without Heavy ORMs | Compile-time checked queries and embedded migrations |
| [ADR 0004](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0004-hybrid-search-lindera-tantivy-fastembed.md) | Hybrid Search Architecture | In-process Tantivy BM25 + Lindera Japanese + FastEmbed vectors |
| [ADR 0005](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0005-react-router-v7-state-and-forms.md) | Frontend State & Forms | TanStack Query + Zustand + React Hook Form + Zod |
| [ADR 0006](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0006-zero-docker-single-machine-windows-deployment.md) | Zero-Docker Windows Deployment | Native process deployment for internal Windows/AWS servers |

---

## Verification & Checks

Before submitting or completing any work, ensure all verification checks pass:

```bash
# Rust checks
cargo clippy --all-targets --all-features
cargo check

# Frontend checks
bun run check     # Biome linting and formatting check
bun run typecheck # TypeScript compiler check
```
