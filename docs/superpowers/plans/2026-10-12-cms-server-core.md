# cms-server-core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish `crates/cms-server-core` as the foundational server infrastructure crate (adopting MedBrains' powerful patterns: trusted-proxy client IP extraction, class-scoped sliding window rate limiter, cursor & offset pagination, regex validation & XSS protection, double-submit CSRF, terminal colored HTTP tracing, and unified AppState).

**Architecture:** Create `crates/cms-server-core` housing zero-route server foundations (`pagination`, `validation`, `http_trace`, `app_state`, and the comprehensive `middleware` suite). Keep `crates/cms-middleware` as a zero-overhead facade re-exporting `cms_server_core` for seamless backwards compatibility.

**Tech Stack:** Rust (Edition 2024, MSRV 1.85), Axum 0.8, Tower 0.5, Tower-HTTP 0.6, Tokio 1.0, SQLx 0.9, ipnet 2.0, regex, serde, serde_json, uuid 1.0, chrono.

## Global Constraints
- Absolute Zero Placeholders: No `todo!()`, `unimplemented!()`, or stubbed functions.
- Rust Edition 2024 with `forbid(unsafe_code)` and strict Clippy compliance (no `unwrap_used`, `panic`, etc.).
- Internal Documentation Platform scope: No billing, no marketing, no public pricing features (per ADR 0001).
- Windows parallel build constraint: Run compilation/tests with `-j 2`.
- Git commits on Windows: Must use `--no-verify` to bypass hanging pre-commit bash scripts.

---

### Task 1: Create `crates/cms-server-core` Crate Scaffolding & Core Types

**Files:**
- Create: `crates/cms-server-core/Cargo.toml`
- Create: `crates/cms-server-core/src/lib.rs`
- Modify: `Cargo.toml:40-46`

**Interfaces:**
- Produces: `cms-server-core` crate available in workspace dependencies.

- [ ] **Step 1: Define `crates/cms-server-core/Cargo.toml`**
  Declare dependencies for Axum, Tower, Tower-HTTP, Tokio, SQLx, Serde, Regex, Ipnet, and internal crates (`cms-config`, `cms-db`, `cms-biz`, `cms-error`, `cms-queue`, `cms-storage`, `cms-search`, `cms-mailer`, `cms-analytics`).

- [ ] **Step 2: Register in root `Cargo.toml`**
  Add `cms-server-core = { path = "crates/cms-server-core" }` under `[workspace.dependencies]`.

- [ ] **Step 3: Create `crates/cms-server-core/src/lib.rs`**
  Export submodules (`pagination`, `validation`, `http_trace`, `middleware`, `app_state`).

- [ ] **Step 4: Verify initial crate check**
  Run: `cargo check -p cms-server-core -j 2`
  Expected: Clean compilation.

- [ ] **Step 5: Commit scaffolding**
  `git add Cargo.toml crates/cms-server-core/Cargo.toml crates/cms-server-core/src/lib.rs`
  `git commit -m "feat(server-core): scaffold cms-server-core crate" --no-verify`

---

### Task 2: Implement Shared Foundation Modules (Pagination, Validation, HTTP Trace)

**Files:**
- Create: `crates/cms-server-core/src/pagination.rs`
- Create: `crates/cms-server-core/src/validation.rs`
- Create: `crates/cms-server-core/src/http_trace.rs`
- Create: `crates/cms-server-core/tests/foundation_test.rs`

**Interfaces:**
- Produces:
  - `Pagination` (`limit()`, `page()`, `offset()`), `PageMeta`, `Paginated<T>`
  - `Cursor` (`limit()`), `CursorPage<T>` (`build()`)
  - `ValidationErrors` (`add()`, `has_errors()`, `into_fields()`), validators (`validate_code`, `validate_name`, `validate_email`, `validate_password`, `validate_no_html`, etc.)
  - `init_colour()`, `terminal_supports_colour()`, status-colored logging for Tower TraceLayer.

- [ ] **Step 1: Write foundation unit tests**
  Create `crates/cms-server-core/tests/foundation_test.rs` testing offset pagination clamping, cursor pagination overfetch trimming, validation rules (code, password complexity, HTML tag rejection), and color initialization.

- [ ] **Step 2: Implement `pagination.rs`**
  Implement `Pagination`, `PageMeta`, `Paginated<T>`, `Cursor`, and `CursorPage<T>` adopted from MedBrains.

- [ ] **Step 3: Implement `validation.rs`**
  Implement `ValidationErrors` and regex validators (code, email, password strength, username, no-HTML anti-XSS, hex color).

- [ ] **Step 4: Implement `http_trace.rs`**
  Implement ANSI color terminal status code mapping, latency formatting, and `NO_COLOR` / `FORCE_COLOR` handling from MedBrains.

- [ ] **Step 5: Run foundation tests**
  Run: `cargo test -p cms-server-core --test foundation_test -j 2`
  Expected: PASS all tests.

- [ ] **Step 6: Commit foundation modules**
  `git add crates/cms-server-core/src/pagination.rs crates/cms-server-core/src/validation.rs crates/cms-server-core/src/http_trace.rs crates/cms-server-core/tests/foundation_test.rs`
  `git commit -m "feat(server-core): implement pagination, validation, and http_trace modules" --no-verify`

---

### Task 3: Implement Enterprise Middleware Suite in `cms-server-core`

**Files:**
- Create: `crates/cms-server-core/src/middleware/mod.rs`
- Create: `crates/cms-server-core/src/middleware/client_ip.rs`
- Create: `crates/cms-server-core/src/middleware/request_id.rs`
- Create: `crates/cms-server-core/src/middleware/rate_limit.rs`
- Create: `crates/cms-server-core/src/middleware/csrf.rs`
- Create: `crates/cms-server-core/src/middleware/ip_restrict.rs`
- Create: `crates/cms-server-core/src/middleware/admin_origin.rs`
- Create: `crates/cms-server-core/src/middleware/locale.rs`
- Create: `crates/cms-server-core/src/middleware/security_headers.rs`
- Create: `crates/cms-server-core/src/middleware/observability.rs`
- Create: `crates/cms-server-core/tests/middleware_test.rs`

**Interfaces:**
- Produces:
  - `ClientIp(pub IpAddr)`, `client_ip_middleware` with trusted proxy validation.
  - `MakeRequestUuid`, `request_id_header()`.
  - `RateLimiter` with `check_and_record_class`.
  - `csrf_middleware` double-submit cookie protection.
  - `ip_in_cidr`, `ip_restrict_middleware`.
  - `AdminOriginLayer`, `SecurityHeadersLayer`, `LocaleLayer`, Prometheus metrics.

- [ ] **Step 1: Write middleware integration tests**
  Test trusted proxy IP extraction, spoofing rejection, CIDR mask matching (IPv4 & IPv6), double-submit CSRF validation, and class-scoped rate limiter in `crates/cms-server-core/tests/middleware_test.rs`.

- [ ] **Step 2: Implement `client_ip.rs` and `request_id.rs`**
  Implement spoofing-resistant client IP extraction with `trusted_proxies: &[ipnet::IpNet]` and UUID v4 request ID generation.

- [ ] **Step 3: Implement `rate_limit.rs`, `csrf.rs`, and `ip_restrict.rs`**
  Implement class-scoped sliding window rate limiter, double-submit cookie CSRF middleware, and CIDR subnet restrictions from MedBrains.

- [ ] **Step 4: Port & enhance `admin_origin.rs`, `locale.rs`, `security_headers.rs`, and `observability.rs`**
  Port CORS origin validation, Accept-Language locale negotiation, HSTS/CSP security headers, and Prometheus scrape metrics into `cms-server-core`.

- [ ] **Step 5: Run middleware tests**
  Run: `cargo test -p cms-server-core --test middleware_test -j 2`
  Expected: PASS all tests.

- [ ] **Step 6: Commit middleware suite**
  `git add crates/cms-server-core/src/middleware/ crates/cms-server-core/tests/middleware_test.rs`
  `git commit -m "feat(server-core): implement enterprise middleware suite (client_ip, csrf, rate_limit, ip_restrict, security_headers)" --no-verify`

---

### Task 4: Unified `AppState` in `cms-server-core` & Facade in `cms-middleware`

**Files:**
- Create: `crates/cms-server-core/src/app_state.rs`
- Modify: `crates/cms-middleware/Cargo.toml`
- Modify: `crates/cms-middleware/src/lib.rs`
- Modify: `apps/api/Cargo.toml`
- Modify: `apps/api/main.rs`

**Interfaces:**
- Produces:
  - `cms_server_core::AppState` containing database pool, config, trusted proxies, rate limiter, and all subsystems.
  - `cms-middleware` cleanly re-exporting `cms_server_core` for 100% backwards compatibility across all existing call sites.

- [ ] **Step 1: Implement `AppState` in `crates/cms-server-core/src/app_state.rs`**
  Combine `Config`, `BizContext`, `Storage`, `SearchEngine`, `AnalyticsStore`, `Mailer`, `EmailRenderer`, `JobQueue`, `trusted_proxies`, and `RateLimiter` into `AppState`.

- [ ] **Step 2: Update `crates/cms-middleware/Cargo.toml` & `src/lib.rs`**
  Add `cms-server-core = { workspace = true }` and re-export `cms_server_core::*` and `cms_server_core::middleware::*` in `cms-middleware/src/lib.rs`.

- [ ] **Step 3: Update `apps/api/main.rs` & `Cargo.toml`**
  Update `apps/api` to use `cms-server-core` directly for `AppState`, `build_cors`, `init_observability`, and middleware stack.

- [ ] **Step 4: Verify server compilation**
  Run: `cargo check -p cms-server -j 2`
  Expected: Clean compilation.

- [ ] **Step 5: Commit unified AppState and facade integration**
  `git add crates/cms-server-core/src/app_state.rs crates/cms-middleware/ apps/api/`
  `git commit -m "feat(server-core): unify AppState in cms-server-core and re-export facade in cms-middleware" --no-verify`

---

### Task 5: End-to-End Verification & Documentation

**Files:**
- Modify: `AGENTS.md`
- Create: `.superpowers/sdd/progress-server-core.md`

- [ ] **Step 1: Run comprehensive tests across server crates**
  Run: `cargo test -p cms-server-core -j 2`
  Run: `cargo check -p cms-api -j 2`
  Run: `cargo check -p cms-server -j 2`

- [ ] **Step 2: Update `AGENTS.md`**
  Document `cms-server-core` as the shared server foundation in `AGENTS.md`.

- [ ] **Step 3: Commit verification and documentation**
  `git add AGENTS.md .superpowers/sdd/`
  `git commit -m "docs: document cms-server-core foundation and enterprise middleware architecture" --no-verify`
