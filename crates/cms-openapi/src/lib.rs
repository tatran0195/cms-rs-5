//! CMS OpenAPI vertical slice
//!
//! Encapsulates OpenAPI document synchronization, spec parsing, and interactive Swagger/ReDoc viewers.

pub mod docs;
pub mod handlers;
pub mod router;
pub mod service;

pub use docs::{serve_openapi_spec, serve_openapi_yaml, serve_redoc, serve_swagger_ui};
pub use handlers::{
    create_openapi_document_handler, delete_openapi_document_handler, get_openapi_content_handler,
    get_openapi_document_handler, list_openapi_documents_handler, parse_openapi_document_handler,
    update_openapi_document_handler,
};
pub use router::router;
pub use service::OpenApiService;
