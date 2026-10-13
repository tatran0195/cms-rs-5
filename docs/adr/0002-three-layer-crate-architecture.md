# ADR 0002: Three-Layer Crate Architecture & Infrastructure Separation

- **Status**: Superseded by [ADR 0007: Modular Monolith Transition to Vertical Slices](0007-vertical-slices-modular-monolith.md)
- **Date**: 2026-10-12
- **Deciders**: Architecture Team

## Context

The backend must support ~25 cohesive documentation domains (pages, projects, assets, branches, comments, deployments, domains, search, git sync, MCP tools, etc.). 

We evaluated two architectural patterns:
1. **Extreme Vertical Micro-Crates** (like MedBrains with 137 crates for 67 full hospital modules): Creating `cms-page`, `cms-branch`, `cms-comment`, `cms-deployment`, etc. as individual crates.
2. **Three-Layer Architecture + Infrastructure Crates**: Layering business logic cleanly horizontally across core domains (`cms-entity` -> `cms-db` -> `cms-biz` -> `cms-api`), while isolating cross-cutting infrastructure into dedicated crates.

## Decision

We adopt the **Three-Layer Architecture with Dedicated Infrastructure Crates**:

1. **Layering Flow**:
   - `cms-entity`: Pure domain types, DTOs, and shared data structures. Zero heavy dependencies.
   - `cms-db`: SQLx queries, schema migrations, and database repository operations. Depends on `cms-entity`.
   - `cms-biz`: Core business logic services, validation, workflow orchestration, and domain rules. Depends on `cms-entity`, `cms-db`.
   - `cms-api`: Axum HTTP routers, request extractors, and response handlers. Depends on `cms-biz`, `cms-entity`, `cms-db`.
   - `cms-server`: Composition root binary (`crates/cms-server`). Assembles all routers, loads configuration, initializes observability, and starts the Axum server.

2. **Dedicated Infrastructure & Cross-Cutting Crates**:
   - `cms-auth`, `cms-authz`, `cms-authz-gate`, `cms-api-keys`, `cms-mfa`, `cms-oauth`: Authentication & access control.
   - `cms-storage`: Pluggable object/file storage (Local file system and AWS S3).
   - `cms-search`: Search indexing and query engine (Lindera Japanese tokenizer, Tantivy BM25, FastEmbed vectors).
   - `cms-queue` & `cms-worker`: Background asynchronous jobs (Memory queue for single node, Redis for distributed).
   - `cms-mailer`: Transactional email delivery via Lettre & MiniJinja.
   - `cms-mcp`: Model Context Protocol server exposing CMS capabilities to AI assistants.
   - `cms-i18n`: Internationalization and translation services.
   - `cms-error` & `cms-config`: Centralized error types (`AppError`) and configuration loading.

## Consequences

- Prevents explosive micro-crate proliferation while maintaining strict separation of concerns.
- Avoids cyclic dependencies between presentation (API), domain rules (Biz), and persistence (DB).
- Allows fast incremental builds for infrastructure crates (`cms-search`, `cms-storage`) that rarely change.
- Keeps domain business logic testable in `cms-biz` without requiring live HTTP servers.
