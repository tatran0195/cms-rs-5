//! Theme Business Logic
//!
//! Domain service for theme management.
//! Accepts `pool: &PgPool` directly; no dependency on `BizContext`.

use cms_db::{PgPool, project::ProjectQueries, theme::ThemeQueries};
use cms_entity::theme::{CreateThemeRequest, ThemeCssVariables, ThemeResponse, UpdateThemeRequest};
use cms_error::AppError;

/// Theme service
#[derive(Debug)]
pub struct ThemeService;

impl ThemeService {
    /// Create a theme
    pub async fn create_theme(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        request: CreateThemeRequest,
    ) -> Result<ThemeResponse, AppError> {
        // Verify project exists
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let theme = ThemeQueries::create(
            pool,
            project_id,
            &request.name,
            request.config.clone().unwrap_or_default(),
            request.is_global.unwrap_or(false),
        )
        .await?;

        Ok(theme.into())
    }

    /// Get a theme
    pub async fn get_theme(
        pool: &PgPool,
        _user_id: &str,
        theme_id: &str,
    ) -> Result<ThemeResponse, AppError> {
        let theme = ThemeQueries::get_by_id(pool, theme_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Theme not found".to_string()))?;

        Ok(theme.into())
    }

    /// List themes for a project
    pub async fn list_themes(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<Vec<ThemeResponse>, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let themes = ThemeQueries::get_by_project(pool, project_id).await?;

        Ok(themes.into_iter().map(|t| t.into()).collect())
    }

    /// Update a theme
    pub async fn update_theme(
        pool: &PgPool,
        _user_id: &str,
        theme_id: &str,
        request: UpdateThemeRequest,
    ) -> Result<ThemeResponse, AppError> {
        let _theme = ThemeQueries::get_by_id(pool, theme_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Theme not found".to_string()))?;

        let updated = ThemeQueries::update(
            pool,
            theme_id,
            request.name.as_deref(),
            request.config.as_ref(),
            request.is_global,
        )
        .await?;

        Ok(updated.into())
    }

    /// Delete a theme
    pub async fn delete_theme(
        pool: &PgPool,
        _user_id: &str,
        theme_id: &str,
    ) -> Result<bool, AppError> {
        let theme = ThemeQueries::get_by_id(pool, theme_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Theme not found".to_string()))?;

        // Cannot delete global theme
        if theme.is_global {
            return Err(AppError::AccessDenied(
                "Cannot delete global theme".to_string(),
            ));
        }

        ThemeQueries::delete(pool, theme_id).await
    }

    /// Get theme CSS variables
    pub async fn get_theme_css(
        pool: &PgPool,
        _user_id: &str,
        theme_id: &str,
    ) -> Result<ThemeCssVariables, AppError> {
        let theme = ThemeQueries::get_by_id(pool, theme_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Theme not found".to_string()))?;

        Ok(theme.into())
    }

    /// Set project theme
    pub async fn set_project_theme(
        pool: &PgPool,
        user_id: &str,
        project_id: &str,
        theme_id: &str,
    ) -> Result<ThemeResponse, AppError> {
        let theme = Self::get_theme(pool, user_id, theme_id).await?;

        cms_db::project::ProjectSettingsQueries::upsert(
            pool,
            project_id,
            Some(theme_id),
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(theme)
    }
}
