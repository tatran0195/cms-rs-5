//! Comment domain business logic.
//!
//! Handles comments on pages, role enforcement, nested replies,
//! and resolution state.

use cms_db::PgPool;
use crate::queries::{CommentQueries, PageQueries};
use cms_entity::{
    comment::{CommentResponse, CommentWithReplies, CreateCommentRequest, UpdateCommentRequest},
    common::{MemberRole, PaginatedResponse},
};
use cms_error::AppError;

/// Business service for Comment operations.
#[derive(Debug)]
pub struct CommentService;

impl CommentService {
    /// Create a new comment on a page
    pub async fn create_comment(
        pool: &PgPool,
        user_id: &str,
        page_id: &str,
        request: CreateCommentRequest,
    ) -> Result<CommentResponse, AppError> {
        let _page = PageQueries::get_by_id(pool, page_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;

        if let Some(parent_id) = &request.parent_id {
            let parent = CommentQueries::get_by_id(pool, parent_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Parent comment not found".to_string()))?;
            if parent.page_id != page_id {
                return Err(AppError::Conflict(
                    "Parent comment does not belong to this page".to_string(),
                ));
            }
        }

        let comment = CommentQueries::create(
            pool,
            page_id,
            Some(user_id),
            None,
            request.parent_id.as_deref(),
            &request.content,
        )
        .await?;

        Ok(comment.into())
    }

    /// Helper to enforce required role using in-memory member role
    fn require_role(member_role: Option<MemberRole>, min_role: MemberRole) -> Result<(), AppError> {
        if let Some(role) = member_role {
            if role >= min_role {
                return Ok(());
            }
        }
        Err(AppError::Forbidden)
    }

    /// Get a comment by ID
    pub async fn get_comment(
        pool: &PgPool,
        user_id: &str,
        comment_id: &str,
    ) -> Result<CommentResponse, AppError> {
        let auth_record = CommentQueries::get_with_auth(pool, comment_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        Self::require_role(auth_record.member_role, MemberRole::Viewer)?;

        Ok(auth_record.comment.into())
    }

    /// List comments for a page
    pub async fn list_comments(
        pool: &PgPool,
        _user_id: &str,
        page_id: &str,
        parent_id: Option<&str>,
        resolved: Option<bool>,
        page: u64,
        page_size: u64,
    ) -> Result<PaginatedResponse<CommentResponse>, AppError> {
        let _page_entity = PageQueries::get_by_id(pool, page_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;

        let limit = i64::try_from(page_size.clamp(1, 1000)).unwrap_or(50);
        let offset = i64::try_from(page.saturating_sub(1))
            .unwrap_or(0)
            .saturating_mul(limit);
        let comments = CommentQueries::get_by_page(
            pool,
            page_id,
            parent_id,
            resolved,
            Some(limit),
            Some(offset),
        )
        .await?;

        let total = CommentQueries::count_by_page(pool, page_id, parent_id, resolved).await?;

        Ok(PaginatedResponse::new(
            comments.into_iter().map(Into::into).collect(),
            total as u64,
            page,
            page_size,
        ))
    }

    /// Update a comment
    pub async fn update_comment(
        pool: &PgPool,
        user_id: &str,
        comment_id: &str,
        expected_project_id: Option<&str>,
        request: UpdateCommentRequest,
    ) -> Result<CommentResponse, AppError> {
        let auth_record = CommentQueries::get_with_auth(pool, comment_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if let Some(expected_pid) = expected_project_id {
            if !auth_record.belongs_to_project(expected_pid) {
                return Err(AppError::NotFound(
                    "Comment not found for this project".to_string(),
                ));
            }
        }

        if auth_record.is_author(user_id) {
            Self::require_role(auth_record.member_role, MemberRole::Viewer)?;
        } else {
            Self::require_role(auth_record.member_role, MemberRole::Admin)?;
        }

        let updated = CommentQueries::update(
            pool,
            comment_id,
            request.content.as_deref(),
            request.resolved,
        )
        .await?;

        Ok(updated.into())
    }

    /// Delete a comment
    pub async fn delete_comment(
        pool: &PgPool,
        user_id: &str,
        comment_id: &str,
        expected_project_id: Option<&str>,
    ) -> Result<bool, AppError> {
        let auth_record = CommentQueries::get_with_auth(pool, comment_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if let Some(expected_pid) = expected_project_id {
            if !auth_record.belongs_to_project(expected_pid) {
                return Err(AppError::NotFound(
                    "Comment not found for this project".to_string(),
                ));
            }
        }

        if auth_record.is_author(user_id) {
            Self::require_role(auth_record.member_role, MemberRole::Viewer)?;
        } else {
            Self::require_role(auth_record.member_role, MemberRole::Admin)?;
        }

        CommentQueries::delete(pool, comment_id).await
    }

    /// Resolve a comment (admin only)
    pub async fn resolve_comment(
        pool: &PgPool,
        user_id: &str,
        comment_id: &str,
    ) -> Result<CommentResponse, AppError> {
        let auth_record = CommentQueries::get_with_auth(pool, comment_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        Self::require_role(auth_record.member_role, MemberRole::Admin)?;

        let updated = CommentQueries::update(pool, comment_id, None, Some(true)).await?;

        Ok(updated.into())
    }

    /// Unresolve a comment (admin only)
    pub async fn unresolve_comment(
        pool: &PgPool,
        user_id: &str,
        comment_id: &str,
    ) -> Result<CommentResponse, AppError> {
        let auth_record = CommentQueries::get_with_auth(pool, comment_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        Self::require_role(auth_record.member_role, MemberRole::Admin)?;

        let updated = CommentQueries::update(pool, comment_id, None, Some(false)).await?;

        Ok(updated.into())
    }

    /// Get comment with replies
    pub async fn get_comment_with_replies(
        pool: &PgPool,
        user_id: &str,
        comment_id: &str,
    ) -> Result<CommentWithReplies, AppError> {
        let auth_record = CommentQueries::get_with_auth(pool, comment_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        Self::require_role(auth_record.member_role, MemberRole::Viewer)?;

        let replies = CommentQueries::get_by_page(
            pool,
            &auth_record.comment.page_id,
            Some(comment_id),
            None,
            None,
            None,
        )
        .await?;

        Ok(CommentWithReplies {
            comment: auth_record.comment.into(),
            replies: replies.into_iter().map(Into::into).collect(),
        })
    }
}
