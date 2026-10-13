# ADR 0007: Modular Monolith Transition to Vertical Slices & Elimination of Horizontal Layers

- **Status**: Accepted
- **Date**: 2026-10-12
- **Deciders**: Architecture Team
- **Supersedes**: [ADR 0002: Three-Layer Crate Architecture & Infrastructure Separation](0002-three-layer-crate-architecture.md)

## Context

Originally under ADR 0002, the CMS platform followed a horizontal 3-layer architecture (`cms-entity` -> `cms-db` -> `cms-biz` -> `cms-api`) with cross-cutting infrastructure crates. As domain functionality grew to encompass content management, project settings, reader access, custom domains, git synchronization, deployments, third-party integrations, themes, onboarding, invitations, and workspace administration:

1. `cms-biz` and `cms-api` became monolithic catch-all "junk drawers" that coupled unrelated business logic and HTTP routing across disparate domains.
2. Changes to a single domain (e.g. custom domain TLS verification) required rebuilding the entire `cms-biz` and `cms-api` dependency trees.
3. Domain ownership was fractured across horizontal layers instead of packaged cohesively in autonomous vertical modules.

## Decision

We evolve the architecture from a layered monolith to a **Modular Monolith composed of Autonomous Vertical Slices** and a dedicated **Composition Root**:

1. **Complete Elimination of `cms-biz` and `cms-api`**:
   - `crates/cms-biz` and `crates/cms-api` are permanently removed from the repository.
   - All business logic services, domain models, and Axum subrouters are encapsulated directly within dedicated vertical slice crates.

2. **Vertical Slice Crates**:
   - `cms-content`: Pages, branch hierarchies, canonical BCP-47 languages, translations, page comments.
   - `cms-projects`: Projects, members, invitations, settings, custom domains & TLS, reader access, addons.
   - `cms-deployments`: Deployment state machine, site build worker jobs, snapshot generation, retry/cancel.
   - `cms-git`: Repository connections (GitHub/GitLab), sync engines, conflict tracking.
   - `cms-integrations`: Third-party webhooks (Slack, Discord, Teams, Zapier), SSRF-protected ping tests.
   - `cms-themes`: Project and global themes, CSS variable extraction.
   - `cms-notifications`: Thread-safe real-time subscriber hub (`NotificationHub`), WebSocket `/ws`, notification REST.
   - `cms-invitations`: Workspace invitations lifecycle, secure token hashing, transactional email outbox dispatch.
   - `cms-onboarding`: Initial admin creation, cryptographic email verification tokens, onboarding status.
   - `cms-reports`: Asynchronous export lifecycle, atomic sequence codes, artifact downloads.
   - `cms-workspaces`: Workspace administration and settings.
   - `cms-admin`: Platform-level administration parity endpoints, takedown operations, funnels.
   - `cms-public`: Unauthenticated reader-facing JSON API, SEO, RSS, sitemaps, changelog.
   - `cms-openapi`: OpenAPI v3 spec generation, Swagger UI, ReDoc.
   - `cms-platform-events`: Platform telemetry, audit queries, health checks.

3. **Shared Foundation & Infrastructure Layer**:
   - `cms-entity`: Pure shared domain types and DTOs with zero internal dependencies.
   - `cms-db`: Exclusively PostgreSQL infrastructure primitives: migrations, connection pool, transaction context & RLS write attribution (`set_write_context`, `set_read_context`), cryptographic hash-chained audit logging (`AuditLogger`), and atomic sequential ID generation (`next_sequence`). Domain-specific queries are permanently relocated to their respective sovereign vertical slices.
   - `cms-server-core`: Zero-route server foundation (pagination, validation, tracing, unified `AppState`, enterprise middleware suite).
   - `cms-auth`: Canonical authentication provider, sessions, credentials, and the unified `AuthUser` extractor used uniformly across all vertical slice route handlers. Per-slice duplicate auth extractors are forbidden and eliminated.

4. **Composition Root (`apps/api`)**:
   - `apps/api` (package `cms-server`) acts as the single composition root.
   - Assembles and mounts all vertical slice routers in `router.rs` (`create_api_router` and `api_router`).
   - Exposes both a library (`cms_server`) for integration testing and a binary (`cms-server`) for production deployment.

## Consequences

- High domain cohesion: All models, business logic, queries, and HTTP endpoints for a domain live together.
- Low coupling: Domain crates depend on shared foundation crates (`cms-server-core`, `cms-auth`, `cms-db`) rather than horizontally across each other.
- Sovereign queries: No backwards-compatibility shims (`pub use cms_db::...`) in domain crates or database crates.
- Unified auth: All vertical slices extract caller identity via canonical `cms_auth::AuthUser`, ensuring consistent Zanzibar authorization context.
- Fast incremental compilation: Editing one domain (such as `cms-git` or `cms-themes`) only recompiles that vertical slice and the composition root.
- Clean integration testing: Integration tests in `apps/api/tests/` verify end-to-end HTTP behavior against the unified router.
