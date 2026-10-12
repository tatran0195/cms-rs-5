//! Member domain models and row representations.

use chrono::{DateTime, Utc};
pub use cms_entity::authz::{AddProjectMemberRequest, ProjectMember, UpdateProjectMemberRequest};
pub use cms_entity::workspace::{
    WorkspaceInvitationItem, WorkspaceInvitationResponse, WorkspaceMemberItem,
    WorkspaceMemberUser, WorkspaceMembersResponse,
};
use sqlx::FromRow;

/// Database representation of a [`ProjectMember`] row.
#[derive(Debug, Clone, FromRow)]
pub struct ProjectMemberRow {
    pub id: String,
    pub project_id: String,
    pub user_id: String,
    pub role: String,
    pub role_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
