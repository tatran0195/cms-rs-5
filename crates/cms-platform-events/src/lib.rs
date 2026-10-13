//! CMS Platform Events vertical slice
//!
//! Encapsulates platform telemetry, trusted server funnel tracking, and database health probes.

pub mod handlers;
pub mod router;
pub mod service;

pub use handlers::{
    create_platform_event_handler, get_health_handler, get_platform_event_handler,
    list_platform_events_handler,
};
pub use router::router;
pub use service::{FunnelEventType, PlatformEventService};
