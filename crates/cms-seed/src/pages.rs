use chrono::Utc;
use cms_error::AppError;
use sqlx::{Postgres, Transaction};

use crate::{projects::SeedProject, users::deterministic_uuid};

/// Seed markdown page specification.
#[derive(Debug, Clone)]
pub struct SeedPage {
    pub id: String,
    pub project_slug: &'static str,
    pub title: &'static str,
    pub slug: &'static str,
    pub path: &'static str,
    pub description: &'static str,
    pub position: i32,
    pub content: &'static str,
}

impl SeedPage {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: deterministic_uuid("page:eng:arch").to_string(),
                project_slug: "engineering-handbook",
                title: "Platform Architecture & Modular Monolith",
                slug: "architecture",
                path: "/architecture",
                description: "Overview of domain vertical slices and core platform subsystems.",
                position: 0,
                content: r#"# Platform Architecture & Modular Monolith

This platform is structured as a **high-performance modular monolith** built in Rust 2024 edition.

## Architectural Principles
1. **Vertical Slices**: Features are encapsulated end-to-end within dedicated crates (e.g. `cms-notifications`, `cms-outbox`).
2. **Transactional Outbox**: Side effects such as emails and webhooks are queued atomically with database mutations.
3. **Strict Reliability**: Zero panics, zero unwrap calls, and compile-time checked SQLx queries.
"#,
            },
            Self {
                id: deterministic_uuid("page:eng:standards").to_string(),
                project_slug: "engineering-handbook",
                title: "Rust Coding Standards & Clippy Gates",
                slug: "coding-standards",
                path: "/coding-standards",
                description: "Guidelines and linter policies for Rust development.",
                position: 1,
                content: r#"# Rust Coding Standards

## Compilation Gates
- Minimum Supported Rust Version (MSRV): **1.85**
- Edition: **2024**
- `unsafe` code is strictly forbidden (`forbid(unsafe_code)`).

## Linting Requirements
- No `unwrap()` or `expect()` in library crates.
- Use structured error propagation via `cms_error::AppError`.
- Format code cleanly via `cargo fmt`.
"#,
            },
            Self {
                id: deterministic_uuid("page:api:overview").to_string(),
                project_slug: "api-reference",
                title: "REST API Overview & Authentication",
                slug: "overview",
                path: "/overview",
                description: "API design conventions, authentication headers, and standard schemas.",
                position: 0,
                content: r#"# REST API Overview

All API endpoints reside under `/api/`.

## Authentication
Requests must include a valid session cookie or an `Authorization: Bearer <token>` header.

## Response Format
Standard JSON responses are returned for both successes and structured errors:
```json
{
  "error": "Not Found",
  "message": "Project with id p_123 does not exist"
}
```
"#,
            },
            Self {
                id: deterministic_uuid("page:api:notifications").to_string(),
                project_slug: "api-reference",
                title: "Real-Time WebSocket & Notification Hub",
                slug: "notifications",
                path: "/notifications",
                description: "Live notification hub and WebSocket connection lifecycle.",
                position: 1,
                content: r#"# Real-Time WebSocket Hub

Connect via WebSocket to receive live user notifications:
`GET /api/notifications/ws`

## Protocol
The hub streams JSON event packets:
- `ping`: Heartbeat event
- `notification_created`: New notification received
- `notification_read`: Notification status updated
- `unread_count`: Real-time unread badge synchronization
"#,
            },
        ]
    }
}

/// Seed documentation pages for existing projects within a transaction.
pub async fn seed_pages(
    tx: &mut Transaction<'_, Postgres>,
    projects: &[SeedProject],
) -> Result<usize, AppError> {
    let pages = SeedPage::defaults();
    let now = Utc::now();
    let mut count = 0;

    for page in &pages {
        let matching_project = projects
            .iter()
            .find(|p| p.slug == page.project_slug)
            .ok_or_else(|| {
                AppError::NotFound(format!("Project slug not found: {}", page.project_slug))
            })?;

        let branch_id =
            deterministic_uuid(&format!("branch:{}:main", matching_project.id)).to_string();

        sqlx::query(
            r#"
            INSERT INTO "Page" (
                id, project_id, branch_id, language_id, parent_id,
                kind, path, slug, title, description, content,
                icon, config, translation_key, position, is_published,
                is_indexed, created_at, updated_at
            ) VALUES (
                $1, $2, $3, NULL, NULL,
                'document', $4, $5, $6, $7, $8,
                'file-text', '{}', NULL, $9, true,
                true, $10, $10
            )
            ON CONFLICT (id) DO UPDATE
            SET title = EXCLUDED.title,
                content = EXCLUDED.content,
                description = EXCLUDED.description,
                position = EXCLUDED.position,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&page.id)
        .bind(&matching_project.id)
        .bind(&branch_id)
        .bind(page.path)
        .bind(page.slug)
        .bind(page.title)
        .bind(page.description)
        .bind(page.content)
        .bind(page.position)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        count += 1;
    }

    Ok(count)
}
