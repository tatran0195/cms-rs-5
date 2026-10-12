# Vertical Slices & Platform Subsystems SDD Progress

Baseline: 8866b6b
Branch: feat/vertical-slices-subsystems
Plan: docs/superpowers/plans/2026-10-12-vertical-slices-subsystems.md

## Tasks
- [x] Task 1: `crates/cms-notifications` Vertical Slice Crate (models, queries, service, WebSocket live hub, Axum router, tests)
- [x] Task 2: Subsystem 1: Transactional Outbox (`crates/cms-outbox`, migrations/0020_outbox.sql, queue_in_tx, worker, handlers, tests)
- [x] Task 3: Subsystem 2: Seed & Dev Fixtures (`crates/cms-seed`, users, workspace, projects, pages, members, tests)
- [x] Task 4: Subsystem 3: Onboarding Verification (`migrations/0021_email_verification_tokens.sql`, token logic, API routes, tests)
- [x] Task 5: Subsystem 4: Workspace Invitations (`migrations/0022_workspace_invitations.sql`, invite logic, API routes, tests)
- [x] Task 6: Subsystem 5: Scheduling (`migrations/0023_scheduled_jobs.sql`, `crates/cms-scheduler`, cron engine, CmsJob scheduled_at, tests)
- [ ] Task 7: Subsystem 6: Report Runs (`migrations/0024_report_runs.sql`, report run lifecycle, API routes, tests)
- [ ] Task 8: Verification & Documentation (all tests pass, clippy passes, AGENTS.md updated)
