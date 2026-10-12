//! Project workspace vertical slice: projects, members, settings, domains, reader access, addons.

use std::sync::Arc;
use axum::Router;
use cms_server_core::app_state::AppState;

pub mod models;
pub mod queries;

pub use models::{
    AddProjectDomainRequest, AddProjectMemberRequest, CreateDomainRequest, CreateProjectRequest,
    DeleteDomainResponse, DeleteProjectResponse, DnsRecord, Domain, DomainResponse,
    DomainVerificationResult, ListDomainsQuery, ListProjectsQuery, ListProjectsResponse, Project,
    ProjectAddon, ProjectAddonResponse, ProjectAuditEvent, ProjectCountResponse,
    ProjectMember, ProjectResponse, ProjectSettings, SpaDomainResponse,
    UpdateDomainRequest, UpdateProjectMemberRequest, UpdateProjectRequest,
    UpdateProjectSettingsRequest, WorkspaceInvitationItem, WorkspaceMemberItem,
    WorkspaceMemberUser, WorkspaceMembersResponse,
};
pub use queries::{
    normalize_hostname, DomainQueries, ProjectAddonQueries, ProjectMemberQueries, ProjectQueries,
    ProjectSettingsQueries,
};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().with_state(state)
}
