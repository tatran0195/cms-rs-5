//! Project business logic decoupled from `BizContext`.

use cms_entity::common::PaginatedResponse;
use cms_error::AppError;
use sqlx::PgPool;

use crate::models::{
    CreateProjectRequest, ListProjectsQuery, ListProjectsResponse, ProjectAddonResponse,
    ProjectResponse, ProjectSettings, UpdateProjectRequest, UpdateProjectSettingsRequest,
};
use crate::queries::{ProjectAddonQueries, ProjectQueries, ProjectSettingsQueries};

/// Project service
#[derive(Debug)]
pub struct ProjectService;

impl ProjectService {
    /// Sanitize a project name into a URL-friendly slug
    #[must_use]
    pub fn sanitize_slug(name: &str) -> String {
        let mut slug = String::new();
        let mut prev_dash = false;
        for c in name.trim().chars().flat_map(char::to_lowercase) {
            if c.is_ascii_alphanumeric() {
                slug.push(c);
                prev_dash = false;
            } else if (c == ' ' || c == '-' || c == '_') && !prev_dash {
                slug.push('-');
                prev_dash = true;
            }
        }
        let trimmed = slug.trim_matches('-');
        if trimmed.is_empty() {
            "project".to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Create a new project atomically along with default branch, language, settings, and owner member.
    pub async fn create_project(
        pool: &PgPool,
        user_id: &str,
        request: CreateProjectRequest,
    ) -> Result<ProjectResponse, AppError> {
        let base_slug = Self::sanitize_slug(&request.name);
        let mut slug = base_slug.clone();
        let mut counter = 1;

        // Atomically insert with collision retry on PostgreSQL unique constraint conflict (23505)
        let project = loop {
            match ProjectQueries::create(
                pool,
                &request.name,
                &slug,
                request.description.as_deref(),
                request.icon.as_deref(),
                request.is_public,
                Some(user_id),
            )
            .await
            {
                Ok(res) => break res,
                Err(AppError::Conflict(_)) => {
                    slug = format!("{base_slug}-{counter}");
                    counter += 1;
                    if counter > 50 {
                        return Err(AppError::Conflict(
                            "Unable to allocate unique project slug after multiple attempts"
                                .to_string(),
                        ));
                    }
                }
                Err(err) => return Err(err),
            }
        };

        Ok(project.into())
    }

    /// Get a project by ID
    pub async fn get_project(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<ProjectResponse, AppError> {
        let project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        Ok(project.into())
    }

    /// Get a public project by project slug
    pub async fn get_public_project(
        pool: &PgPool,
        project_slug: &str,
    ) -> Result<ProjectResponse, AppError> {
        let project = ProjectQueries::get_by_slug(pool, project_slug)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;
        if !project.is_public {
            return Err(AppError::NotFound("Project not found".to_string()));
        }
        Ok(project.into())
    }

    /// Get a project by slug
    pub async fn get_project_by_slug(
        pool: &PgPool,
        _user_id: &str,
        project_slug: &str,
    ) -> Result<ProjectResponse, AppError> {
        let project = ProjectQueries::get_by_slug(pool, project_slug)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        Ok(project.into())
    }

    /// Update a project
    pub async fn update_project(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        request: UpdateProjectRequest,
    ) -> Result<ProjectResponse, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let updated = ProjectQueries::update(
            pool,
            project_id,
            request.name.as_deref(),
            request.slug.as_deref(),
            request.description.as_deref(),
            request.icon.as_deref(),
            request.is_public,
            request.config.as_ref(),
        )
        .await?;

        Ok(updated.into())
    }

    /// Delete a project
    pub async fn delete_project(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<bool, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        ProjectQueries::delete(pool, project_id).await
    }

    /// List all projects
    #[allow(clippy::cast_possible_wrap)]
    pub async fn list_projects(
        pool: &PgPool,
        _user_id: &str,
        query: ListProjectsQuery,
        page: u64,
        page_size: u64,
    ) -> Result<ListProjectsResponse, AppError> {
        let offset = page.saturating_sub(1) * page_size;
        let projects = ProjectQueries::get_all(
            pool,
            query.is_public,
            query.search.as_deref(),
            Some(page_size as i64),
            Some(offset as i64),
        )
        .await?;

        let total = ProjectQueries::count(
            pool,
            query.is_public,
            query.search.as_deref(),
        )
        .await?;

        Ok(PaginatedResponse::new(
            projects.into_iter().map(Into::into).collect(),
            total as u64,
            page,
            page_size,
        ))
    }

    /// List all projects accessible to a user
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub async fn list_all_projects_for_user(
        pool: &PgPool,
        user_id: &str,
        page: u64,
        page_size: u64,
    ) -> Result<ListProjectsResponse, AppError> {
        let projects = ProjectQueries::list_by_user(pool, user_id).await?;
        let total = projects.len() as u64;
        let offset = (page.saturating_sub(1) * page_size) as usize;
        let paged: Vec<ProjectResponse> = projects
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .map(Into::into)
            .collect();

        Ok(PaginatedResponse::new(paged, total, page, page_size))
    }

    /// Get project settings
    pub async fn get_project_settings(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<ProjectSettings, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let settings = ProjectSettingsQueries::get(pool, project_id)
            .await?
            .unwrap_or_else(|| ProjectSettings {
                project_id: project_id.to_string(),
                theme: None,
                default_language: None,
                custom_domain: None,
                search_enabled: true,
                comments_enabled: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });

        Ok(settings)
    }

    /// Update project settings
    pub async fn update_project_settings(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        request: UpdateProjectSettingsRequest,
    ) -> Result<ProjectSettings, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let settings = ProjectSettingsQueries::upsert(
            pool,
            project_id,
            request.theme.as_deref(),
            request.default_language.as_deref(),
            request.custom_domain.as_deref(),
            request.search_enabled,
            request.comments_enabled,
        )
        .await?;

        Ok(settings)
    }

    /// List project addons
    pub async fn list_project_addons(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<Vec<ProjectAddonResponse>, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let addons = ProjectAddonQueries::get_by_project(pool, project_id).await?;

        Ok(addons.into_iter().map(Into::into).collect())
    }

    /// Create a project addon
    pub async fn create_project_addon(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        addon_type: &str,
        config: serde_json::Value,
        is_enabled: bool,
    ) -> Result<ProjectAddonResponse, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let addon =
            ProjectAddonQueries::create(pool, project_id, addon_type, config, is_enabled)
                .await?;

        Ok(addon.into())
    }

    /// Update a project addon
    pub async fn update_project_addon(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        addon_id: &str,
        config: Option<serde_json::Value>,
        is_enabled: Option<bool>,
    ) -> Result<ProjectAddonResponse, AppError> {
        let addon = ProjectAddonQueries::get_by_id(pool, addon_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Addon not found".to_string()))?;

        if addon.project_id != project_id {
            return Err(AppError::NotFound(
                "Addon not found for this project".to_string(),
            ));
        }

        let updated = ProjectAddonQueries::update(pool, addon_id, config, is_enabled).await?;

        Ok(updated.into())
    }

    /// Delete a project addon
    pub async fn delete_project_addon(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        addon_id: &str,
    ) -> Result<bool, AppError> {
        let addon = ProjectAddonQueries::get_by_id(pool, addon_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Addon not found".to_string()))?;

        if addon.project_id != project_id {
            return Err(AppError::NotFound(
                "Addon not found for this project".to_string(),
            ));
        }

        ProjectAddonQueries::delete(pool, addon_id).await
    }
}
