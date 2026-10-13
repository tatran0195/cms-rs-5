# Rule: Crate Boundaries & Layering Architecture

## Context

The CMS backend repository is organized as a Cargo workspace with distinct horizontal layers and specialized infrastructure crates.

---

## 🏗️ Layering Flow (Strict Dependency Hierarchy)

```
cms-server (Composition Root Binary)
      ↓
   cms-api (HTTP Axum Routers & Handlers)
      ↓
   cms-biz (Business Logic Services & Orchestration)
      ↓
   cms-db (Database SQLx Repositories & Migrations)
      ↓
  cms-entity (Domain Types, DTOs & Models)
```

### Dependency Rules:
1. `cms-entity` has ZERO dependencies on other internal crates. It contains pure data structures, enums, and serde derivations.
2. `cms-db` depends only on `cms-entity`, `sqlx`, and database-related primitives. It NEVER depends on `cms-biz` or `cms-api`.
3. `cms-biz` depends on `cms-entity`, `cms-db`, and infrastructure crates (`cms-storage`, `cms-search`, etc.). It NEVER depends on `cms-api` or HTTP concepts.
4. `cms-api` depends on `cms-biz`, `cms-entity`, `cms-db`, `cms-auth`, `cms-middleware`. It translates HTTP requests into business logic service calls.
5. `cms-server` is the composition root binary. It initializes configuration, starts background workers, connects the database pool, and mounts Axum routers.

---

## ⚙️ Infrastructure Crates

Dedicated infrastructure crates must remain isolated and modular:
- `cms-storage`: File/asset storage abstraction (Local directory, S3).
- `cms-search`: Search indexing and query engine (Lindera, Tantivy, FastEmbed).
- `cms-queue` & `cms-worker`: Asynchronous background task processing.
- `cms-mailer`: Transactional email sending.
- `cms-mcp`: Model Context Protocol server.
- `cms-auth`, `cms-authz`, `cms-api-keys`: Security, sessions, JWTs, and API key verification.

Do NOT introduce circular dependencies between infrastructure crates.

---

## 🚫 Avoid Anti-Patterns

1. **No Monolithic Leaks**: Handlers in `cms-api` must NOT execute raw SQL queries directly; delegate data operations to `cms-biz` or `cms-db`.
2. **No HTTP in Business Logic**: Services in `cms-biz` must NOT take Axum extractors (`Json<T>`, `Path<T>`, `HeaderMap`) or return Axum `Response`.
3. **No Dead Crates**: Keep `Cargo.toml` `[workspace.members]` and `default-members` strictly in sync with the filesystem.
