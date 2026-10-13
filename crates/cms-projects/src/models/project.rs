//! Project models and database row representations.

use chrono::{DateTime, Utc};
pub use cms_entity::project::{
    CreateProjectRequest, DeleteProjectResponse, ListProjectsQuery, ListProjectsResponse, Project,
    ProjectAddon, ProjectAddonResponse, ProjectAuditEvent, ProjectCountResponse, ProjectResponse,
    ProjectSettings, UpdateProjectRequest, UpdateProjectSettingsRequest,
};
use sqlx::FromRow;

/// Database representation of a project row
#[derive(Debug, Clone, FromRow)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub is_public: bool,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            icon: row.icon,
            is_public: row.is_public,
            config: row.config,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Database representation of a project settings row
#[derive(Debug, Clone, FromRow)]
pub struct ProjectSettingsRow {
    pub project_id: String,
    pub theme: Option<String>,
    pub default_language: Option<String>,
    pub custom_domain: Option<String>,
    pub search_enabled: bool,
    pub comments_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectSettingsRow> for ProjectSettings {
    fn from(row: ProjectSettingsRow) -> Self {
        Self {
            project_id: row.project_id,
            theme: row.theme,
            default_language: row.default_language,
            custom_domain: row.custom_domain,
            search_enabled: row.search_enabled,
            comments_enabled: row.comments_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Database representation of a project addon row
#[derive(Debug, Clone, FromRow)]
pub struct ProjectAddonRow {
    pub id: String,
    pub project_id: String,
    pub addon_type: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectAddonRow> for ProjectAddon {
    fn from(row: ProjectAddonRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            addon_type: row.addon_type,
            config: row.config,
            is_enabled: row.is_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
