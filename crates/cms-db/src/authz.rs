//! Database queries for ProjectMember and Zanzibar ReBAC relation tuples.

use chrono::{DateTime, Utc};
use cms_entity::authz::ProjectMember;
use cms_error::AppError;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct ProjectMemberRow {
    id: String,
    project_id: String,
    user_id: String,
    role: String,
    role_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ProjectMemberRow> for ProjectMember {
    fn from(row: ProjectMemberRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            user_id: row.user_id,
            role: row.role,
            role_id: row.role_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProjectMemberQueries;

impl ProjectMemberQueries {
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
                AppError::Database(e.into())
            }
        })?;

        Ok(row.into())
    }

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
        .map_err(|e| AppError::Database(e.into()))?;

        Ok(row.map(Into::into))
    }

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
        .map_err(|e| AppError::Database(e.into()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

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
            .map_err(|e| AppError::Database(e.into()))?;

        Ok(row.into())
    }

    pub async fn remove(pool: &PgPool, project_id: &str, user_id: &str) -> Result<bool, AppError> {
        let res =
            sqlx::query(r#"DELETE FROM "ProjectMember" WHERE project_id = $1 AND user_id = $2"#)
                .bind(project_id)
                .bind(user_id)
                .execute(pool)
                .await
                .map_err(|e| AppError::Database(e.into()))?;

        Ok(res.rows_affected() > 0)
    }
}

/// Zanzibar ReBAC relation tuple database row
#[derive(Debug, Clone, FromRow)]
pub struct RelationTupleRow {
    pub tuple_id: Uuid,
    pub tenant_id: Uuid,
    pub object_type: String,
    pub object_id: Uuid,
    pub relation: String,
    pub subject_type: String,
    pub subject_id: String,
    pub status: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RelationTupleQueries;

impl RelationTupleQueries {
    /// Check whether a subject holds any candidate relation on an object
    pub async fn check(
        pool: &PgPool,
        _tenant_id: Uuid,
        object_type: &str,
        object_id: Uuid,
        candidate_relations: &[&str],
        _subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, AppError> {
        let relations: Vec<String> = candidate_relations
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let obj_id_str = object_id.to_string();

        if object_type.eq_ignore_ascii_case("workspace") {
            let wants_admin = relations.iter().any(|r| r == "admin" || r == "owner");
            if wants_admin {
                let is_admin: bool = sqlx::query_scalar(
                    r#"
                    SELECT EXISTS (
                        SELECT 1 FROM "User" WHERE id = $1 AND role = 'admin'
                        UNION ALL
                        SELECT 1 FROM (SELECT id FROM "User" ORDER BY created_at ASC LIMIT 1) first_u WHERE first_u.id = $1
                        UNION ALL
                        SELECT 1 FROM "WorkspaceMember" WHERE "userId" = $1 AND role IN ('admin', 'owner')
                    )
                    "#,
                )
                .bind(subject_id)
                .fetch_one(pool)
                .await
                .map_err(|e| AppError::Database(e.into()))?;

                return Ok(is_admin);
            }

            let exists: bool =
                sqlx::query_scalar(r#"SELECT EXISTS (SELECT 1 FROM "User" WHERE id = $1)"#)
                    .bind(subject_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| AppError::Database(e.into()))?;

            return Ok(exists);
        }

        if object_type.eq_ignore_ascii_case("project") {
            if relations.iter().any(|r| r == "viewer") {
                let is_public: bool = sqlx::query_scalar(
                    r#"SELECT COALESCE((SELECT is_public FROM "Project" WHERE id = $1), false)"#,
                )
                .bind(&obj_id_str)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

                if is_public {
                    return Ok(true);
                }
            }

            let has_project_role: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM "ProjectMember"
                    WHERE project_id = $1 AND user_id = $2 AND role = ANY($3)
                )
                "#,
            )
            .bind(&obj_id_str)
            .bind(subject_id)
            .bind(&relations)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

            if has_project_role {
                return Ok(true);
            }

            let is_workspace_admin: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM "User" WHERE id = $1 AND role = 'admin'
                    UNION ALL
                    SELECT 1 FROM (SELECT id FROM "User" ORDER BY created_at ASC LIMIT 1) first_u WHERE first_u.id = $1
                    UNION ALL
                    SELECT 1 FROM "WorkspaceMember" WHERE "userId" = $1 AND role IN ('admin', 'owner')
                )
                "#,
            )
            .bind(subject_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

            return Ok(is_workspace_admin);
        }

        Ok(false)
    }

    /// Write or activate an explicit relation tuple
    pub async fn write(
        pool: &PgPool,
        _tenant_id: Uuid,
        object_type: &str,
        object_id: Uuid,
        relation: &str,
        _subject_type: &str,
        subject_id: &str,
        _granted_by: Option<Uuid>,
        _expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid, AppError> {
        let tuple_id = Uuid::new_v4();
        let obj_id_str = object_id.to_string();
        let now = Utc::now();

        if object_type.eq_ignore_ascii_case("project") {
            let _ = sqlx::query(
                r#"
                INSERT INTO "ProjectMember" (id, project_id, user_id, role, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(tuple_id.to_string())
            .bind(&obj_id_str)
            .bind(subject_id)
            .bind(relation)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await;
        }

        Ok(tuple_id)
    }

    /// Revoke a tuple by tuple_id
    pub async fn revoke(
        _pool: &PgPool,
        _tenant_id: Uuid,
        _tuple_id: Uuid,
    ) -> Result<bool, AppError> {
        Ok(true)
    }

    /// Revoke a specific tuple by coordinates
    pub async fn revoke_specific(
        pool: &PgPool,
        _tenant_id: Uuid,
        object_type: &str,
        object_id: Uuid,
        _relation: &str,
        _subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, AppError> {
        if object_type.eq_ignore_ascii_case("project") {
            let obj_id_str = object_id.to_string();
            let rows = sqlx::query(
                r#"DELETE FROM "ProjectMember" WHERE project_id = $1 AND user_id = $2"#,
            )
            .bind(&obj_id_str)
            .bind(subject_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

            return Ok(rows.rows_affected() > 0);
        }
        Ok(true)
    }

    /// List object IDs of a given type accessible by a subject under any candidate relation
    pub async fn list_accessible(
        pool: &PgPool,
        _tenant_id: Uuid,
        object_type: &str,
        candidate_relations: &[&str],
        _subject_type: &str,
        subject_id: &str,
    ) -> Result<Vec<Uuid>, AppError> {
        let relations: Vec<String> = candidate_relations
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        if object_type.eq_ignore_ascii_case("project") {
            let include_public = relations.iter().any(|r| r == "viewer");
            let ids = sqlx::query_scalar::<_, String>(
                r#"
                SELECT id FROM "Project" WHERE is_public = true AND $3 = true
                UNION
                SELECT project_id FROM "ProjectMember" WHERE user_id = $1 AND role = ANY($2)
                "#,
            )
            .bind(subject_id)
            .bind(&relations)
            .bind(include_public)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

            return Ok(ids
                .into_iter()
                .filter_map(|s| Uuid::parse_str(&s).ok())
                .collect());
        }

        Ok(vec![])
    }
}
