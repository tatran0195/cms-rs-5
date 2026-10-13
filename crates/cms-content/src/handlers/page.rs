//! Page HTTP handlers for content vertical slice.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_entity::{
    common::{Id, PaginatedResponse},
    page::{CreatePageRequest, ListPagesQuery, PageListItem, PageResponse, UpdatePageRequest},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::{handlers::auth::ContentUser, services::page::PageService};

/// List pages for a project and branch
pub async fn list_pages_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Query(query): Query<ListPagesQuery>,
) -> Result<Json<PaginatedResponse<PageListItem>>, AppError> {
    let result = PageService::list_pages(state.db(), &auth.user_id, query, 1, 20).await?;
    Ok(Json(result))
}

/// Create a new page
pub async fn create_page_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Json(request): Json<CreatePageRequest>,
) -> Result<Json<PageResponse>, AppError> {
    let project_id = request.project_id.clone();
    let branch_id = request.branch_id.clone();
    let page =
        PageService::create_page(state.db(), &auth.user_id, &project_id, &branch_id, request)
            .await?;

    Ok(Json(page))
}

/// Get a specific page by ID
pub async fn get_page_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(page_id): Path<Id>,
) -> Result<Json<PageResponse>, AppError> {
    let page = PageService::get_page(state.db(), &auth.user_id, &page_id).await?;
    Ok(Json(page))
}

/// Update an existing page
pub async fn update_page_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(page_id): Path<Id>,
    Json(request): Json<UpdatePageRequest>,
) -> Result<Json<PageResponse>, AppError> {
    let page = PageService::update_page(state.db(), &auth.user_id, &page_id, request).await?;
    Ok(Json(page))
}

/// Delete a page
pub async fn delete_page_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(page_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    PageService::delete_page(state.db(), &auth.user_id, &page_id).await?;
    Ok(Json(serde_json::json!({"success": true, "id": page_id})))
}
