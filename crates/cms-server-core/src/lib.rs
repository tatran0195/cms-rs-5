//! CMS Server Core — Shared server foundation and enterprise middleware.
//!
//! Provides zero-route server foundations:
//! - Pagination (`Pagination`, `CursorPage`)
//! - Validation (`ValidationErrors`, regex validators, anti-XSS)
//! - HTTP tracing (`init_colour`, status-painted logs)
//! - Enterprise middleware (`client_ip`, `rate_limit`, `csrf`, `ip_restrict`, `security_headers`)
//! - Unified `AppState`

pub mod http_trace;
pub mod pagination;
pub mod validation;

// Submodules to be enabled as implemented:
// pub mod middleware;
// pub mod app_state;
// pub use app_state::AppState;
