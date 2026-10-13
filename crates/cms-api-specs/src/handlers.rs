//! OpenAPI handlers

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use cms_auth::AuthExtractor;
use cms_entity::{
    common::{Id, PaginatedResponse},
    openapi::{
        CreateOpenApiDocumentRequest, ListOpenApiDocumentsQuery, OpenApiDocumentResponse,
        OpenApiParsingResult, ParseOpenApiDocumentRequest, UpdateOpenApiDocumentRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::service::OpenApiService;

/// List OpenAPI documents
pub async fn list_openapi_documents_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListOpenApiDocumentsQuery>,
) -> Result<Json<PaginatedResponse<OpenApiDocumentResponse>>, AppError> {
    let project_id = query.project_id.as_deref().unwrap_or("");
    let documents =
        OpenApiService::list_documents(state.db(), &auth.user.id, project_id).await?;
    let total = documents.len() as u64;

    Ok(Json(PaginatedResponse::new(documents, total, 1, 20)))
}

/// Create an OpenAPI document
pub async fn create_openapi_document_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(request): Json<CreateOpenApiDocumentRequest>,
) -> Result<Json<OpenApiDocumentResponse>, AppError> {
    let document =
        OpenApiService::create_document(state.db(), &auth.user.id, request).await?;

    Ok(Json(document))
}

/// Get an OpenAPI document
pub async fn get_openapi_document_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(document_id): Path<Id>,
) -> Result<Json<OpenApiDocumentResponse>, AppError> {
    let document =
        OpenApiService::get_document(state.db(), &auth.user.id, &document_id).await?;

    Ok(Json(document))
}

/// Update an OpenAPI document
pub async fn update_openapi_document_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(document_id): Path<Id>,
    Json(request): Json<UpdateOpenApiDocumentRequest>,
) -> Result<Json<OpenApiDocumentResponse>, AppError> {
    let document =
        OpenApiService::update_document(state.db(), &auth.user.id, &document_id, request)
            .await?;

    Ok(Json(document))
}

/// Delete an OpenAPI document
pub async fn delete_openapi_document_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(document_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    OpenApiService::delete_document(state.db(), &auth.user.id, &document_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "id": document_id}),
    ))
}

/// Parse an OpenAPI document
pub async fn parse_openapi_document_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(_document_id): Path<Id>,
    Json(request): Json<ParseOpenApiDocumentRequest>,
) -> Result<Json<OpenApiParsingResult>, AppError> {
    let result = OpenApiService::parse_document(state.db(), &auth.user.id, request).await?;

    Ok(Json(result))
}

/// Get OpenAPI document content
pub async fn get_openapi_content_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(document_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let content =
        OpenApiService::get_document_content(state.db(), &auth.user.id, &document_id).await?;

    Ok(Json(serde_json::json!({"content": content})))
}
