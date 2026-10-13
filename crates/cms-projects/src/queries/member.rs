//! Project member database queries

use chrono::Utc;
use cms_error::AppError;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::{ProjectMember, ProjectMemberRow};

/// Project member queries
#[derive(Debug)]
pub struct ProjectMemberQueries;

impl ProjectMemberQueries {
    /// Add a member to a project
    pub async fn create(
        pool: &PgPool,
        project_id: &str,
        user_id: &str,
        role: &str,
        role_id: Option<&str>,
    ) -> Result<ProjectMember, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let row = sqlx::query_as::<_, ProjectMemberRow>(
            r#"
            INSERT INTO "ProjectMember" (id, project_id, user_id, role, role_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(user_id)
        .bind(role)
        .bind(role_id)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::Conflict("User is already a member of this project".to_string())
            } else {
                AppError::Database(e)
            }
        })?;

        Ok(row.into())
    }

    /// Get project member by user ID and project ID
    pub async fn get_by_user_and_project(
        pool: &PgPool,
        user_id: &str,
        project_id: &str,
    ) -> Result<Option<ProjectMember>, AppError> {
        let row = sqlx::query_as::<_, ProjectMemberRow>(
            r#"SELECT * FROM "ProjectMember" WHERE user_id = $1 AND project_id = $2"#,
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row.map(Into::into))
    }

    /// List all members of a project
    pub async fn list_by_project(
        pool: &PgPool,
        project_id: &str,
    ) -> Result<Vec<ProjectMember>, AppError> {
        let rows = sqlx::query_as::<_, ProjectMemberRow>(
            r#"SELECT * FROM "ProjectMember" WHERE project_id = $1 ORDER BY created_at ASC"#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Update a member's role
    #[allow(clippy::useless_let_if_seq)]
    pub async fn update_role(
        pool: &PgPool,
        project_id: &str,
        user_id: &str,
        role: Option<&str>,
        role_id: Option<&str>,
    ) -> Result<ProjectMember, AppError> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE \"ProjectMember\" SET ");
        let mut has_updates = false;

        if let Some(role) = role {
            qb.push("role = ");
            qb.push_bind(role);
            has_updates = true;
        }
        if let Some(role_id) = role_id {
            if has_updates {
                qb.push(", ");
            }
            qb.push("role_id = ");
            qb.push_bind(role_id);
            has_updates = true;
        }

        if has_updates {
            qb.push(", updated_at = ");
            qb.push_bind(Utc::now());
        }

        qb.push(" WHERE project_id = ");
        qb.push_bind(project_id);
        qb.push(" AND user_id = ");
        qb.push_bind(user_id);
        qb.push(" RETURNING *");

        let row = qb
            .build_query_as::<ProjectMemberRow>()
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(row.into())
    }

    /// Remove a member from a project
    pub async fn remove(pool: &PgPool, project_id: &str, user_id: &str) -> Result<bool, AppError> {
        let res =
            sqlx::query(r#"DELETE FROM "ProjectMember" WHERE project_id = $1 AND user_id = $2"#)
                .bind(project_id)
                .bind(user_id)
                .execute(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(res.rows_affected() > 0)
    }

    /// Transfer project ownership atomically
    pub async fn transfer_ownership(
        pool: &PgPool,
        project_id: &str,
        current_owner_id: &str,
        new_owner_id: &str,
    ) -> Result<(), AppError> {
        let mut tx = pool.begin().await.map_err(AppError::Database)?;
        let now = Utc::now();

        // Demote current owner to admin
        sqlx::query(
            r#"UPDATE "ProjectMember" SET role = 'admin', updated_at = $3 WHERE project_id = $1 AND user_id = $2"#,
        )
        .bind(project_id)
        .bind(current_owner_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        // Promote new owner to owner
        sqlx::query(
            r#"UPDATE "ProjectMember" SET role = 'owner', updated_at = $3 WHERE project_id = $1 AND user_id = $2"#,
        )
        .bind(project_id)
        .bind(new_owner_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }
}
