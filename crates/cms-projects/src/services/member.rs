//! Project member service decoupled from `BizContext`.

use std::collections::HashMap;

use chrono::{Duration, Utc};
use cms_entity::workspace::{
    WorkspaceInvitationResponse, WorkspaceMemberItem, WorkspaceMemberUser,
    WorkspaceMembersResponse,
};
use cms_error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::ProjectMember;
use crate::queries::ProjectMemberQueries;

/// Member service
#[derive(Debug)]
pub struct MemberService;

impl MemberService {
    /// List all members for a project
    pub async fn list_members(
        pool: &PgPool,
        project_id: &str,
    ) -> Result<WorkspaceMembersResponse, AppError> {
        let proj_members = ProjectMemberQueries::list_by_project(pool, project_id).await?;

        let uids: Vec<&str> = proj_members.iter().map(|m| m.user_id.as_str()).collect();
        let users = if uids.is_empty() {
            Vec::new()
        } else {
            cms_db::auth::UserQueries::get_by_ids(pool, &uids)
                .await
                .unwrap_or_default()
        };
        let user_map: HashMap<String, cms_entity::auth::User> =
            users.into_iter().map(|u| (u.id.clone(), u)).collect();

        let items: Vec<WorkspaceMemberItem> = proj_members
            .into_iter()
            .map(|m| {
                let (user_name, user_email, user_image) = user_map.get(&m.user_id).map_or_else(
                    || (None, String::new(), None),
                    |u| (u.name.clone(), u.email.clone(), u.image.clone()),
                );
                WorkspaceMemberItem {
                    id: m.id,
                    user_id: m.user_id.clone(),
                    role: m.role,
                    created_at: m.created_at,
                    user: WorkspaceMemberUser {
                        id: m.user_id,
                        name: user_name,
                        email: user_email,
                        image: user_image,
                    },
                }
            })
            .collect();

        Ok(WorkspaceMembersResponse {
            members: items,
            invitations: Vec::new(),
        })
    }

    /// Invite a member to a project
    pub async fn invite_member(
        pool: &PgPool,
        project_id: &str,
        email: &str,
        role: &str,
        role_id: Option<&str>,
    ) -> Result<WorkspaceInvitationResponse, AppError> {
        if let Ok(Some(existing_user)) = cms_db::auth::UserQueries::get_by_email(pool, email).await {
            let _ = ProjectMemberQueries::create(
                pool,
                project_id,
                &existing_user.id,
                role,
                role_id,
            )
            .await;
        }

        let invitation_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires = now + Duration::days(7);

        Ok(WorkspaceInvitationResponse {
            id: invitation_id,
            email: email.to_string(),
            role: role.to_string(),
            token: None,
            expires_at: expires.to_rfc3339(),
            created_at: now.to_rfc3339(),
        })
    }

    /// Update a member's role
    pub async fn update_member_role(
        pool: &PgPool,
        project_id: &str,
        user_id: &str,
        role: Option<&str>,
        role_id: Option<&str>,
    ) -> Result<ProjectMember, AppError> {
        ProjectMemberQueries::update_role(pool, project_id, user_id, role, role_id).await
    }

    /// Remove a member from a project
    pub async fn remove_member(
        pool: &PgPool,
        project_id: &str,
        user_id: &str,
    ) -> Result<bool, AppError> {
        ProjectMemberQueries::remove(pool, project_id, user_id).await
    }

    /// Transfer project ownership atomically
    pub async fn transfer_ownership(
        pool: &PgPool,
        project_id: &str,
        current_owner_id: &str,
        new_owner_id: &str,
    ) -> Result<(), AppError> {
        ProjectMemberQueries::transfer_ownership(pool, project_id, current_owner_id, new_owner_id).await
    }
}
