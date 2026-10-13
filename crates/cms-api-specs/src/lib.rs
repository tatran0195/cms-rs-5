//! CMS API Specifications vertical slice
//!
//! Encapsulates stored OpenAPI document synchronization, spec parsing, and validation.

pub mod handlers;
pub mod router;
pub mod service;

pub use handlers::{
    create_openapi_document_handler, delete_openapi_document_handler, get_openapi_content_handler,
    get_openapi_document_handler, list_openapi_documents_handler, parse_openapi_document_handler,
    update_openapi_document_handler,
};
pub use router::router;
pub use service::OpenApiService;
