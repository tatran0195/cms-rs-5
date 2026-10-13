//! Project workspace vertical slice: projects, members, settings, domains, reader access, addons.

use std::sync::Arc;

use axum::Router;
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod models;
pub mod queries;
pub mod services;

pub use handlers::{
    ProjectUser, addons_router, core_router, domains_router, members_router, project_router,
    reader_access_router, tls_check_router,
};
pub use models::{
    AddProjectDomainRequest, AddProjectMemberRequest, CreateDomainRequest, CreateProjectRequest,
    DeleteDomainResponse, DeleteProjectResponse, DnsRecord, Domain, DomainResponse,
    DomainVerificationResult, ListDomainsQuery, ListProjectsQuery, ListProjectsResponse, Project,
    ProjectAddon, ProjectAddonResponse, ProjectAuditEvent, ProjectCountResponse, ProjectMember,
    ProjectResponse, ProjectSettings, SpaDomainResponse, UpdateDomainRequest,
    UpdateProjectMemberRequest, UpdateProjectRequest, UpdateProjectSettingsRequest,
    WorkspaceInvitationItem, WorkspaceInvitationResponse, WorkspaceMemberItem, WorkspaceMemberUser,
    WorkspaceMembersResponse,
};
pub use queries::{
    ProjectAddonQueries, ProjectMemberQueries, ProjectQueries, ProjectSettingsQueries,
};
pub use services::{DomainService, MemberService, ProjectService, ReaderAccessService};

pub fn router(state: Arc<AppState>) -> Router {
    project_router(state)
}
