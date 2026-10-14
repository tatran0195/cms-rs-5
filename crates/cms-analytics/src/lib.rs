//! CMS Analytics Vertical Slice
//!
//! Autonomous domain crate handling event tracking, metrics aggregation,
//! page view counters, and project dashboards.

pub mod handlers;
pub mod router;
pub mod services;

pub use router::router;
pub use services::{AnalyticsService, process_analytics_job};
