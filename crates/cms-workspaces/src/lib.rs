//! CMS Workspaces vertical slice
//!
//! Encapsulates company workspace settings, workspace members, invitations, and analytics.

pub mod handlers;
pub mod router;

pub use handlers::{
    cancel_workspace_invitation_handler, delete_workspace_handler, get_workspace_analytics_handler,
    get_workspace_settings_handler, invite_workspace_member_handler,
    list_workspace_members_handler, remove_workspace_member_handler,
    transfer_workspace_ownership_handler, update_workspace_member_role_handler,
    update_workspace_settings_handler,
};
pub use router::{members_router, workspace_router};
