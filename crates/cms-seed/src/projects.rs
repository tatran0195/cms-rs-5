use chrono::Utc;
use cms_error::AppError;
use sqlx::{Postgres, Transaction};

use crate::users::deterministic_uuid;

/// Seed project specification.
#[derive(Debug, Clone)]
pub struct SeedProject {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub is_public: bool,
}

impl SeedProject {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: deterministic_uuid("project:eng-handbook").to_string(),
                name: "Engineering Handbook".to_string(),
                slug: "engineering-handbook".to_string(),
                description: "Core architectural decisions, engineering values, and workflows."
                    .to_string(),
                is_public: false,
            },
            Self {
                id: deterministic_uuid("project:api-reference").to_string(),
                name: "API Reference".to_string(),
                slug: "api-reference".to_string(),
                description: "Comprehensive internal REST, WebSocket, and MCP API contracts."
                    .to_string(),
                is_public: true,
            },
        ]
    }
}

/// Seed demo projects, default branches, and project settings within a transaction.
pub async fn seed_projects(tx: &mut Transaction<'_, Postgres>) -> Result<Vec<SeedProject>, AppError> {
    let projects = SeedProject::defaults();
    let now = Utc::now();

    for p in &projects {
        sqlx::query(
            r#"
            INSERT INTO "Project" (id, name, slug, description, icon, is_public, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'book', $5, '{}', $6, $6)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                description = EXCLUDED.description,
                is_public = EXCLUDED.is_public,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&p.id)
        .bind(&p.name)
        .bind(&p.slug)
        .bind(&p.description)
        .bind(p.is_public)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        // Default Branch: "main"
        let branch_id = deterministic_uuid(&format!("branch:{}:main", p.id)).to_string();
        sqlx::query(
            r#"
            INSERT INTO "Branch" (id, project_id, name, slug, description, is_default, is_protected, created_at, updated_at)
            VALUES ($1, $2, 'main', 'main', 'Default production branch', true, true, $3, $3)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                is_default = true,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&branch_id)
        .bind(&p.id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        // Project settings
        sqlx::query(
            r#"
            INSERT INTO "ProjectSettings" (project_id, theme, default_language, custom_domain, search_enabled, comments_enabled, created_at, updated_at)
            VALUES ($1, 'system', 'en', NULL, true, true, $2, $2)
            ON CONFLICT (project_id) DO UPDATE
            SET search_enabled = true,
                comments_enabled = true,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&p.id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(projects)
}
