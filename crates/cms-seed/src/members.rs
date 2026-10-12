use chrono::Utc;
use cms_error::AppError;
use sqlx::{Postgres, Transaction};

use crate::{projects::SeedProject, users::{deterministic_uuid, SeedUser}};

/// Link seed users to projects with designated roles.
pub async fn seed_members(
    tx: &mut Transaction<'_, Postgres>,
    projects: &[SeedProject],
) -> Result<usize, AppError> {
    let users = SeedUser::defaults();
    let now = Utc::now();
    let mut count = 0;

    for project in projects {
        for user in &users {
            let role = match user.role.as_str() {
                "admin" => "admin",
                _ if user.email.contains("alice") => "editor",
                _ => "viewer",
            };

            let member_id =
                deterministic_uuid(&format!("member:{}:{}", project.id, user.id)).to_string();

            sqlx::query(
                r#"
                INSERT INTO "ProjectMember" (id, project_id, user_id, role, role_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, NULL, $5, $5)
                ON CONFLICT (id) DO UPDATE
                SET role = EXCLUDED.role,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(&member_id)
            .bind(&project.id)
            .bind(&user.id)
            .bind(role)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(AppError::Database)?;

            count += 1;
        }
    }

    Ok(count)
}
