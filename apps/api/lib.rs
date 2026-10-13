//! CMS Server Composition Root Library
//!
//! Provides the composed application router for production execution and integration testing.

pub mod router;

pub use router::{api_router, create_api_router};
