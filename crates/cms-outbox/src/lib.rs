//! `cms-outbox` — Transactional Outbox Pattern for Resilient Message Delivery.
//!
//! Provides durable at-least-once asynchronous event delivery with:
//! - Transactional queuing (`queue_in_tx`)
//! - Concurrent worker drainage with `FOR UPDATE SKIP LOCKED`
//! - Exponential retry backoff with randomized jitter
//! - Dead Letter Queue (`outbox_dlq`) routing for exhausted or permanent errors
//! - Abandoned worker lock reaping

pub mod backoff;
pub mod handler;
pub mod handlers;
pub mod metrics;
pub mod queue;
pub mod worker;

pub use backoff::{BackoffConfig, calculate_backoff};
pub use handler::{Handler, HandlerCtx, HandlerError, Registry};
pub use handlers::{EmailHandler, WebhookHandler};
pub use queue::{
    OutboxRecord, OutboxRow, dispatch_email_in_tx, dispatch_in_tx, queue, queue_batch_in_tx,
    queue_in_tx,
};
pub use worker::{OutboxWorker, OutboxWorkerConfig};
