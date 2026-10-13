//! CMS Worker Library
//!
//! Background job processing worker implementation.
//! It consumes jobs from the configured queue backend (Redis or in-memory).

use std::sync::Arc;

pub use app_state::WorkerState;
use cms_error::AppError;
use cms_infra::queue::JobQueue;
use tracing::{debug, error, info};

/// Start job consumers
pub async fn start_consumers(
    job_queue: Arc<dyn JobQueue>,
    state: Arc<WorkerState>,
) -> Result<Vec<tokio::task::JoinHandle<()>>, AppError> {
    let (_tx, rx) = tokio::sync::watch::channel(false);
    start_consumers_with_shutdown(job_queue, state, rx).await
}

/// Start job consumers with graceful shutdown support
#[allow(clippy::unused_async)]
pub async fn start_consumers_with_shutdown(
    job_queue: Arc<dyn JobQueue>,
    state: Arc<WorkerState>,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<Vec<tokio::task::JoinHandle<()>>, AppError> {
    let config = &state.config.queue;
    let num_workers = config.workers;

    info!("Starting {} job consumer(s)", num_workers);
    let mut handles = Vec::with_capacity(num_workers);

    for i in 0..num_workers {
        let queue = Arc::clone(&job_queue);
        let state = Arc::clone(&state);
        let mut worker_shutdown = shutdown_rx.clone();

        let handle = tokio::spawn(async move {
            let consumer_name = format!("worker-{i}");
            if let Err(e) = run_consumer(queue, state, consumer_name, &mut worker_shutdown).await {
                error!("Consumer {} failed: {}", i, e);
            }
        });
        handles.push(handle);
    }

    Ok(handles)
}

/// Run a single job consumer with graceful shutdown handling
pub async fn run_consumer(
    job_queue: Arc<dyn JobQueue>,
    state: Arc<WorkerState>,
    consumer_name: String,
    shutdown_rx: &mut tokio::sync::watch::Receiver<bool>,
) -> Result<(), AppError> {
    info!("Consumer {} started", consumer_name);

    loop {
        if *shutdown_rx.borrow() {
            info!("Consumer {} shutting down gracefully", consumer_name);
            break;
        }

        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Consumer {} received shutdown signal", consumer_name);
                    break;
                }
            }
            res = job_queue.consume(&consumer_name) => {
                match res {
                    Ok(job) => {
                        debug!(
                            "Consumer {} processing job: {:?}",
                            consumer_name, job.job_type
                        );

                        // Process the job — nack on failure so it can be retried
                        match process_job(&job, Arc::clone(&state)).await {
                            Ok(()) => {
                                if let Err(e) = job_queue.ack(&job.id).await {
                                    error!("Failed to acknowledge job {}: {}", job.id.0, e);
                                }
                            }
                            Err(e) => {
                                error!(
                                    "Failed to process job {} (type={:?}): {}",
                                    job.id.0, job.job_type, e
                                );
                                if let Err(nack_err) = job_queue.nack(&job.id, &e.to_string()).await {
                                    error!(
                                        "Failed to nack job {} after error: {}",
                                        job.id.0, nack_err
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Consumer {} error: {}", consumer_name, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Process a single job
pub async fn process_job(
    job: &cms_infra::queue::JobEnvelope,
    state: Arc<WorkerState>,
) -> Result<(), AppError> {
    use cms_infra::queue::JobType;

    match job.job_type {
        JobType::Analytics => {
            cms_analytics::process_analytics_job(&state.db, &job.payload).await
        }
        JobType::Email => {
            cms_infra::mailer::process_email_job_with_renderer(
                Arc::clone(&state.mailer),
                &state.email_renderer,
                &job.payload,
            )
            .await
        }
        JobType::Export => {
            cms_reports::process_export_job(&state.db, Arc::clone(&state.storage), &job.payload).await
        }
        JobType::Git => cms_git::process_git_job(&state.db, &job.payload).await,
        JobType::Publish => {
            cms_deployments::process_deployment_job(&state.db, Arc::clone(&state.storage), &job.payload)
                .await
        }
        JobType::Search => {
            cms_search::process_search_job(&state.db, Arc::clone(&state.search), &job.payload)
                .await
        }
        JobType::Usage => Ok(()),
        JobType::Reaper => {
            cms_infra::queue::process_reaper_job(&state.db, Arc::clone(&state.job_queue), &job.payload)
                .await
        }
    }
}

/// Worker application state
pub mod app_state {
    use std::sync::Arc;

    use cms_db::PgPool;
    use cms_error::AppError;
    pub use cms_infra::mailer::{
        Email, EmailRenderer, Mailer, NoopMailer, SmtpMailer,
        create_mailer_with_noop_fallback as create_mailer,
    };
    use cms_search::{SearchEngine, create_search_engine};
    use cms_infra::{
        config::Config,
        queue::JobQueue,
        storage::{Storage, create_storage},
    };

    #[derive(Clone)]
    pub struct WorkerState {
        pub config: Arc<Config>,
        pub db: PgPool,
        pub storage: Arc<dyn Storage>,
        pub search: Arc<dyn SearchEngine>,
        pub mailer: Arc<dyn Mailer>,
        pub email_renderer: EmailRenderer,
        pub job_queue: Arc<dyn JobQueue>,
    }

    impl std::fmt::Debug for WorkerState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("WorkerState")
                .field("config", &self.config)
                .finish_non_exhaustive()
        }
    }

    impl WorkerState {
        pub async fn new(config: &Config, job_queue: Arc<dyn JobQueue>) -> Result<Self, AppError> {
            let db = cms_db::create_pool(&config.database.url).await?;
            Self::with_pool(config, job_queue, db).await
        }

        pub async fn with_pool(
            config: &Config,
            job_queue: Arc<dyn JobQueue>,
            db: PgPool,
        ) -> Result<Self, AppError> {
            let storage: Arc<dyn Storage> = create_storage(&config.storage).await?.into();
            let search = create_search_engine(&config.search)?;
            let mailer = create_mailer(config.mailer.as_ref())?;
            let email_renderer = EmailRenderer::embedded().map_err(AppError::internal)?;

            Ok(Self {
                config: Arc::new(config.clone()),
                db,
                storage,
                search,
                mailer,
                email_renderer,
                job_queue,
            })
        }

        /// Build a worker over the API server's shared pools and adapters so
        /// single-binary deployments execute jobs rather than marking them done.
        pub async fn from_app_state(
            app: &cms_server_core::app_state::AppState,
        ) -> Result<Self, AppError> {
            Ok(Self {
                config: Arc::clone(&app.config),
                db: app.biz_context.pool.clone(),
                storage: Arc::clone(&app.storage),
                search: Arc::clone(&app.search_engine),
                mailer: Arc::clone(&app.mailer),
                email_renderer: app.email_renderer.clone(),
                job_queue: Arc::clone(&app.job_queue),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use cms_infra::{config::MailerConfig, mailer::Mailer};

    use crate::app_state::*;

    #[tokio::test]
    async fn test_noop_mailer_send() {
        let mailer = NoopMailer;
        let result = mailer
            .send_email("test@example.com", "Test Subject", "Test Body")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_smtp_mailer_validation() {
        // Missing smtp_host should error
        let config = MailerConfig {
            smtp_host: None,
            smtp_port: 587,
            smtp_username: None,
            smtp_password: None,
            smtp_use_tls: false,
            smtp_plain_no_tls: false,
            from_email: Some("admin@example.com".to_string()),
            from_name: Some("CMS Admin".to_string()),
            logo_url: None,
        };

        let result = SmtpMailer::new(&config);
        assert!(result.is_err());

        // Valid configuration should construct correctly
        let valid_config = MailerConfig {
            smtp_host: Some("smtp.example.com".to_string()),
            smtp_port: 587,
            smtp_username: Some("user".to_string()),
            smtp_password: Some("pass".to_string()),
            smtp_use_tls: false,
            smtp_plain_no_tls: false,
            from_email: Some("admin@example.com".to_string()),
            from_name: Some("CMS Admin".to_string()),
            logo_url: None,
        };

        let valid_result = SmtpMailer::new(&valid_config);
        assert!(valid_result.is_ok());
    }
}
