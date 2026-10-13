//! CMS Platform Admin vertical slice
//!
//! Encapsulates platform operator stats, health checks, metrics, user lifecycle management,
//! and site takedown/restore operations.

pub mod handlers;
pub mod parity;
pub mod router;

pub use handlers::{get_system_health_handler, get_system_metrics_handler, get_system_stats_handler};
pub use router::router;
