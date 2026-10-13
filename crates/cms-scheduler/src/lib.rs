//! CMS Scheduling Subsystem
//!
//! Provides a durable, cron-based scheduling engine for recurring maintenance,
//! sync, and cleanup jobs backed by PostgreSQL and the CMS `JobQueue`.

pub mod cron;
pub mod registry;
pub mod runner;

pub use cron::{next_run_time, normalize_cron_expr, parse_schedule};
pub use registry::{CronJobSpec, JobRegistry};
pub use runner::{SchedulerConfig, SchedulerHandle, SchedulerRunner};

use std::sync::Arc;
use cms_db::PgPool;
use cms_error::AppError;
use cms_infra::queue::JobQueue;

/// High-level facade for the CMS Scheduler subsystem
#[derive(Debug)]
pub struct Scheduler;

impl Scheduler {
    /// Synchronize the given registry (or default registry) into the database
    pub async fn sync_registry(pool: &PgPool, registry: Option<&JobRegistry>) -> Result<(), AppError> {
        let default_reg = JobRegistry::default_registry();
        let reg = registry.unwrap_or(&default_reg);
        reg.sync_to_db(pool).await
    }

    /// Spawn the scheduler worker background task
    pub fn spawn(
        pool: PgPool,
        queue: Arc<dyn JobQueue>,
        config: Option<SchedulerConfig>,
    ) -> SchedulerHandle {
        SchedulerRunner::spawn(pool, queue, config.unwrap_or_default())
    }
}
