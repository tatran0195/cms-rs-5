//! Auth helper and permission authorization helpers for content vertical slice.

use cms_auth::AuthUser;
use cms_authz::Relation;
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

/// Authorize a page action using Zanzibar `ReBAC` parent linkage traversal.
/// Returns resolved parent `project_id` or `NotFound` to defend against existence oracles.
pub async fn authorize_page(
    state: &AppState,
    auth: &AuthUser,
    page_id: &str,
    relation: Relation,
) -> Result<Uuid, AppError> {
    let page_uuid =
        Uuid::parse_str(page_id).map_err(|_| AppError::NotFound("Page not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        cms_authz::gate::ParentLink::Page(page_uuid),
        relation,
    )
    .await
}

/// Authorize a branch action using Zanzibar `ReBAC` parent linkage traversal.
pub async fn authorize_branch(
    state: &AppState,
    auth: &AuthUser,
    branch_id: &str,
    relation: Relation,
) -> Result<Uuid, AppError> {
    let branch_uuid = Uuid::parse_str(branch_id)
        .map_err(|_| AppError::NotFound("Branch not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        cms_authz::gate::ParentLink::Branch(branch_uuid),
        relation,
    )
    .await
}
