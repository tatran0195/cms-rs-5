//! Search HTTP route handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use cms_auth::AuthExtractor;
use cms_entity::{
    common::{ApiResponse, Id, PaginatedResponse, SuccessResponse},
    search::{
        ListSearchIndexRunsQuery, ReindexRequest, SearchIndexRunResponse, SearchRequest,
        SearchResponse,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::services::SearchService;

/// Perform a search query across indexed documents
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    _auth: AuthExtractor,
    Query(request): Query<SearchRequest>,
) -> Result<Json<SearchResponse>, AppError> {
    let project_id = request.project_id.clone();
    let result = SearchService::search(
        state.db(),
        Arc::clone(&state.search_engine),
        &project_id,
        request,
    )
    .await?;

    Ok(Json(result))
}

/// Trigger background reindexing for a project
pub async fn reindex_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(request): Json<ReindexRequest>,
) -> Result<Json<ApiResponse<SuccessResponse>>, AppError> {
    SearchService::reindex(
        state.db(),
        Arc::clone(&state.search_engine),
        &auth.user.id,
        request,
    )
    .await?;

    Ok(Json(ApiResponse::new(SuccessResponse {
        success: true,
        message: Some("Reindexing started".to_string()),
    })))
}

/// List search indexing run history
pub async fn list_search_index_runs_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListSearchIndexRunsQuery>,
) -> Result<Json<PaginatedResponse<SearchIndexRunResponse>>, AppError> {
    let result = SearchService::list_index_runs(state.db(), &auth.user.id, query).await?;

    Ok(Json(result))
}

/// Get detailed status of a specific search index run
pub async fn get_search_index_run_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(run_id): Path<Id>,
) -> Result<Json<SearchIndexRunResponse>, AppError> {
    let run = SearchService::get_index_run(state.db(), &auth.user.id, &run_id).await?;

    Ok(Json(run))
}

/// Get overall search status and metrics for a project
pub async fn get_search_status_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status = SearchService::get_search_status(state.db(), &auth.user.id, &project_id).await?;

    Ok(Json(status))
}
