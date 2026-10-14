//! Comment HTTP handlers for content vertical slice.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_auth::AuthUser;
use cms_entity::{
    comment::{
        CommentResponse, CommentWithReplies, CreateCommentRequest, ListCommentsQuery,
        ResolveCommentRequest, UpdateCommentRequest,
    },
    common::{Id, PaginatedResponse},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::services::comment::CommentService;

/// List comments
pub async fn list_comments_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<PaginatedResponse<CommentResponse>>, AppError> {
    let page_id = query.page_id.as_deref().unwrap_or("");
    let page_size = query.limit.unwrap_or(20).max(1) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;
    let result = CommentService::list_comments(
        state.db(),
        &auth.user_id,
        page_id,
        query.parent_id.as_deref(),
        query.resolved,
        page,
        page_size,
    )
    .await?;

    Ok(Json(result))
}

/// Create a new comment
pub async fn create_comment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let page_id = request.page_id.clone();
    let comment =
        CommentService::create_comment(state.db(), &auth.user_id, &page_id, request).await?;

    Ok(Json(comment))
}

/// Get a comment with its replies
pub async fn get_comment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(comment_id): Path<Id>,
) -> Result<Json<CommentWithReplies>, AppError> {
    let comment =
        CommentService::get_comment_with_replies(state.db(), &auth.user_id, &comment_id).await?;

    Ok(Json(comment))
}

/// Update a comment
pub async fn update_comment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(comment_id): Path<Id>,
    Json(request): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let comment =
        CommentService::update_comment(state.db(), &auth.user_id, &comment_id, None, request)
            .await?;

    Ok(Json(comment))
}

/// Delete a comment
pub async fn delete_comment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(comment_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    CommentService::delete_comment(state.db(), &auth.user_id, &comment_id, None).await?;

    Ok(Json(serde_json::json!({"success": true, "id": comment_id})))
}

/// Resolve or unresolve a comment
pub async fn resolve_comment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(comment_id): Path<Id>,
    Json(request): Json<ResolveCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let comment = if request.resolved {
        CommentService::resolve_comment(state.db(), &auth.user_id, &comment_id).await?
    } else {
        CommentService::unresolve_comment(state.db(), &auth.user_id, &comment_id).await?
    };

    Ok(Json(comment))
}

/// List comments for a specific page
pub async fn list_page_comments_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(page_id): Path<Id>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<PaginatedResponse<CommentResponse>>, AppError> {
    let page_size = query.limit.unwrap_or(20).max(1) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;
    let result = CommentService::list_comments(
        state.db(),
        &auth.user_id,
        &page_id,
        query.parent_id.as_deref(),
        query.resolved,
        page,
        page_size,
    )
    .await?;

    Ok(Json(result))
}
