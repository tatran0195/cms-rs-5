# ADR 0003: Compile-Time Checked SQLx Without Heavy ORMs

- **Status**: Accepted
- **Date**: 2026-10-12
- **Deciders**: Architecture Team

## Context

For database persistence in Rust, teams typically choose between:
1. Full ORMs (e.g. Diesel, SeaORM).
2. Query builders with runtime verification.
3. Raw database drivers with compile-time SQL verification (SQLx).

Heavy ORMs introduce complex macro abstractions, obscure generated SQL queries, make complex relational queries difficult to optimize, and often lead to N+1 query patterns.

## Decision

We use **SQLx with compile-time verified queries** and **no ORM**:

1. **Direct SQL with Compile-Time Guarantees**:
   - All queries use `sqlx::query!`, `sqlx::query_as!`, or `sqlx::query_scalar!`.
   - Queries are checked against database schema during `cargo sqlx prepare`.
   - `.sqlx/` offline metadata is committed to the repository; regular builds/CI/CD run with `SQLX_OFFLINE=true`.
2. **Schema & Migrations**:
   - Migrations are embedded at compile time via `sqlx::migrate!()` in `cms-db`.
   - Migration scripts live in `crates/cms-db/migrations/`.
   - Database schema is explicitly managed via standard PostgreSQL SQL files.
3. **Workspace Isolation**:
   - Every workspace-scoped table has a `workspace_id` column.
   - Handlers and services must ensure tenant/workspace boundaries are verified on every query.

## Consequences

- Full control over executed SQL and execution plans.
- Eliminates runtime SQL syntax errors and schema mismatch bugs before deployment.
- High performance without ORM mapping overhead.
- Requires developers to write explicit SQL migrations and run `make prepare-sqlx` when modifying queries.
