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

pub use backoff::{calculate_backoff, BackoffConfig};
pub use handler::{Handler, HandlerCtx, HandlerError, Registry};
pub use handlers::{EmailHandler, WebhookHandler};
pub use queue::{queue, queue_batch_in_tx, queue_in_tx, OutboxRecord, OutboxRow};
pub use worker::{OutboxWorker, OutboxWorkerConfig};
