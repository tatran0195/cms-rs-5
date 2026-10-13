//! OpenAPI router

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use cms_server_core::app_state::AppState;

use crate::handlers::{
    create_openapi_document_handler, delete_openapi_document_handler, get_openapi_content_handler,
    get_openapi_document_handler, list_openapi_documents_handler, parse_openapi_document_handler,
    update_openapi_document_handler,
};

/// Create the OpenAPI router
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/documents", get(list_openapi_documents_handler))
        .route("/documents", post(create_openapi_document_handler))
        .route("/documents/{id}", get(get_openapi_document_handler))
        .route("/documents/{id}", put(update_openapi_document_handler))
        .route("/documents/{id}", delete(delete_openapi_document_handler))
        .route(
            "/documents/{id}/parse",
            post(parse_openapi_document_handler),
        )
        .route("/documents/{id}/content", get(get_openapi_content_handler))
        .with_state(state)
}
