# Rule: Rust Backend Coding Standards & Clippy Rules

## Context

The CMS backend is written in Rust (Edition 2024, MSRV 1.85) using Axum 0.8, Tower 0.5, and Tokio.
All code must adhere to strict safety, error handling, and linting rules.

---

## 🚫 Denied Practices & Compile-Time Prohibitions

The following are strictly **forbidden** and will fail CI/CD:

1. **`unsafe` Code**:
   - `unsafe` code is **strictly forbidden** (`forbid(unsafe_code)`).
2. **Panic and Crashing Macros**:
   - **Never** use `unwrap()`, `expect()`, `panic!()`, `todo!()`, or `unimplemented!()`.
   - Propagate errors using `?` returning `Result<T, AppError>`.
3. **Debug & Standard Output Macros**:
   - **Never** leave `dbg!()`, `println!()`, or `eprintln!()` in backend crates.
   - Use structured logging via `tracing::info!`, `tracing::warn!`, `tracing::error!`, or `tracing::debug!`.
4. **Wildcard & Sloppy Imports**:
   - `wildcard_imports` (`use foo::*`) are **denied**. Import items explicitly.
5. **Inefficient Conversions**:
   - Deny `inefficient_to_string` and unnecessary allocations.
6. **Clone on Ref-Counted Pointers**:
   - Deny `clone_on_ref_ptr` (use `Arc::clone(&x)` instead of `x.clone()`).

---

## 🛡️ Error Handling Discipline

1. **Library Crates**:
   - Use `thiserror` to define strongly typed, domain-specific error enums in library crates (`cms-db`, `cms-biz`, `cms-storage`, etc.).
2. **Application Boundaries & Composition Root**:
   - Use `cms_error::AppError` at Axum handler and HTTP boundaries.
   - Map domain errors cleanly into HTTP status codes (`400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`, `409 Conflict`, `500 Internal Server Error`).
   - Do NOT leak internal database connection strings, credentials, or internal panic details to HTTP clients.

---

## 💾 SQL & Database Standards

1. **Compile-Time Verified Queries**:
   - All queries must be compile-time checked using `sqlx::query!`, `sqlx::query_as!`, or `sqlx::query_scalar!`.
   - No runtime query construction with untyped string concatenation.
2. **Offline Mode**:
   - Keep `.sqlx/` metadata committed. Normal builds use `SQLX_OFFLINE=true`.
   - Run `cargo sqlx prepare` only against a migrated local/staging database.
3. **Workspace Isolation**:
   - Workspace boundaries must be explicitly checked on every entity query (`WHERE workspace_id = $1`).

---

## 🧹 Code Cleanliness & Formatting

- Format with `cargo fmt`.
- Strict Clippy: run `cargo clippy --all-targets --all-features` with zero warnings.
- Delete dead code immediately: do NOT leave commented-out blocks, unused types, or unused imports.
