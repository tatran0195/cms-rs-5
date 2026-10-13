//! Unified Application State for CMS Server.
//!
//! Houses shared platform context passed to all route handlers:
//! - Configuration & Business context
//! - Storage & Search engine
//! - Queue & Mailer
//! - Host resolution invalidation
//! - Trusted proxy CIDR cache
//! - Class-scoped sliding window rate limiter

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use cms_error::AppError;
use cms_infra::{
    config::Config,
    mailer::{EmailRenderer, Mailer},
    queue::JobQueue,
    search::SearchEngine,
    storage::Storage,
};

use crate::middleware::rate_limit::SlidingRateLimiter;

/// Business context with database pool and access control
#[derive(Clone)]
pub struct BizContext {
    pub pool: sqlx::PgPool,
    pub authz: Arc<cms_authz::AuthzState>,
}

impl std::fmt::Debug for BizContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BizContext")
            .field("pool", &"PgPool")
            .field("authz", &"AuthzState")
            .finish()
    }
}

impl BizContext {
    pub fn new(pool: sqlx::PgPool, authz: Arc<cms_authz::AuthzState>) -> Self {
        Self { pool, authz }
    }

    pub fn authz(&self) -> &Arc<cms_authz::AuthzState> {
        &self.authz
    }
}

/// Application state shared across all HTTP handlers and services.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Arc<Config>,
    /// Business context with database pool and access control
    pub biz_context: BizContext,
    /// Storage backend
    pub storage: Arc<dyn Storage>,
    /// Job queue
    pub job_queue: Arc<dyn JobQueue>,
    /// Search engine
    pub search_engine: Arc<dyn SearchEngine>,
    /// Outbound email delivery used by auth and workflow handlers
    pub mailer: Arc<dyn Mailer>,
    /// Transactional email renderer with embedded localized catalog
    pub email_renderer: EmailRenderer,
    /// Generation counter used to invalidate cached host-to-project resolutions
    pub host_resolution_generation: Arc<AtomicU64>,
    /// Trusted proxy CIDRs for X-Forwarded-For validation
    pub trusted_proxies: Arc<Vec<ipnet::IpNet>>,
    /// In-memory sliding window rate limiter
    pub sliding_rate_limiter: Arc<SlidingRateLimiter>,
}

impl AppState {
    /// Access the underlying PostgreSQL connection pool.
    #[inline]
    #[must_use]
    pub const fn db(&self) -> &sqlx::PgPool {
        &self.biz_context.pool
    }

    /// Returns a reference to the authorization engine state.
    #[inline]
    #[must_use]
    pub const fn authz(&self) -> &Arc<cms_authz::AuthzState> {
        &self.biz_context.authz
    }

    /// Returns a reference to the authorization backend trait object.
    #[inline]
    #[must_use]
    pub fn authz_backend(&self) -> &dyn cms_authz::AuthzBackend {
        self.biz_context.authz.backend.as_ref()
    }

    /// Parse trusted proxy CIDRs from environment variable `TRUSTED_PROXIES`.
    #[must_use]
    pub fn load_trusted_proxies_from_env() -> Vec<ipnet::IpNet> {
        std::env::var("TRUSTED_PROXIES").map_or_else(
            |_| Vec::new(),
            |val| {
                val.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| s.parse::<ipnet::IpNet>().ok())
                    .collect()
            },
        )
    }

    /// Create `AppState` from full configuration.
    pub async fn from_config(config: &Config) -> Result<Self, AppError> {
        config.validate()?;
        let pool = cms_db::create_pool(&config.database.url).await?;
        cms_db::run_migrations(&pool).await?;
        let storage_box = cms_infra::storage::create_storage(&config.storage).await?;
        let storage: Arc<dyn Storage> = Arc::from(storage_box);
        let job_queue =
            cms_infra::queue::create_job_queue_with_pool(&config.queue, pool.clone()).await?;
        let search_engine = cms_infra::search::create_search_engine(&config.search)?;
        let authz = Arc::new(cms_authz::AuthzState::new(
            pool.clone(),
            config.auth.system_admin_emails.clone(),
        ));
        let biz_context = BizContext::new(pool, authz);
        let mailer = cms_infra::mailer::create_mailer(config.mailer.as_ref())?;
        let email_renderer = EmailRenderer::embedded().map_err(AppError::internal)?;
        let trusted_proxies = Arc::new(Self::load_trusted_proxies_from_env());
        let sliding_rate_limiter = Arc::new(SlidingRateLimiter::new());
        cms_infra::init();

        Ok(Self {
            config: Arc::new(config.clone()),
            biz_context,
            storage,
            job_queue,
            search_engine,
            mailer,
            email_renderer,
            host_resolution_generation: Arc::new(AtomicU64::new(0)),
            trusted_proxies,
            sliding_rate_limiter,
        })
    }

    /// Create a new `AppState` with provided base dependencies (using default mailer and proxies).
    pub fn new(
        config: Config,
        biz_context: BizContext,
        storage: Arc<dyn Storage>,
        job_queue: Arc<dyn JobQueue>,
        search_engine: Arc<dyn SearchEngine>,
    ) -> Result<Self, AppError> {
        Self::new_with_mailer(
            config,
            biz_context,
            storage,
            job_queue,
            search_engine,
            Arc::new(cms_infra::mailer::UnconfiguredMailer),
        )
    }

    /// Create state with an explicitly configured mail provider.
    pub fn new_with_mailer(
        config: Config,
        biz_context: BizContext,
        storage: Arc<dyn Storage>,
        job_queue: Arc<dyn JobQueue>,
        search_engine: Arc<dyn SearchEngine>,
        mailer: Arc<dyn Mailer>,
    ) -> Result<Self, AppError> {
        let email_renderer = EmailRenderer::embedded().map_err(AppError::internal)?;
        Ok(Self::new_with_mailer_and_renderer(
            config,
            biz_context,
            storage,
            job_queue,
            search_engine,
            mailer,
            email_renderer,
        ))
    }

    /// Create state with explicitly configured mail provider and email renderer.
    pub fn new_with_mailer_and_renderer(
        config: Config,
        biz_context: BizContext,
        storage: Arc<dyn Storage>,
        job_queue: Arc<dyn JobQueue>,
        search_engine: Arc<dyn SearchEngine>,
        mailer: Arc<dyn Mailer>,
        email_renderer: EmailRenderer,
    ) -> Self {
        let trusted_proxies = Arc::new(Self::load_trusted_proxies_from_env());
        let sliding_rate_limiter = Arc::new(SlidingRateLimiter::new());
        Self {
            config: Arc::new(config),
            biz_context,
            storage,
            job_queue,
            search_engine,
            mailer,
            email_renderer,
            host_resolution_generation: Arc::new(AtomicU64::new(0)),
            trusted_proxies,
            sliding_rate_limiter,
        }
    }

    /// Invalidate cached host-to-project resolutions after a domain change.
    pub fn invalidate_host_resolution_cache(&self) {
        self.host_resolution_generation
            .fetch_add(1, Ordering::AcqRel);
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &"Config")
            .field("biz_context", &"BizContext")
            .field("trusted_proxies", &self.trusted_proxies.len())
            .field(
                "host_resolution_generation",
                &self.host_resolution_generation.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}
