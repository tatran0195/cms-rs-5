//! Platform telemetry and funnel event tracking (operational admin concern).

pub mod handlers;
pub mod router;
pub mod service;

pub use router::router;
pub use service::{FunnelEventType, PlatformEventService};
