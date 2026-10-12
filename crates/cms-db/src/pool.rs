//! PostgreSQL connection pool configuration and factory.
//!
//! The `after_release` hook resets all `app.*` GUCs so a session-level
//! context set by one request never leaks into the next connection reuse.
//!
//! # Production incident note
//! Without `after_release`, `app.workspace_id` outlives the request that set
//! it and is inherited by whoever gets the connection next. Under workspace
//! scoping this silently returns another workspace's data — no error, no
//! warning, just wrong results. The `RESET` chain prevents this.
//! Use `sqlx::raw_sql` (simple protocol) not `sqlx::query` (extended/prepared)
//! because Postgres refuses multiple commands in a prepared statement.

use std::{str::FromStr, time::Duration};

use cms_error::AppError;
use log::LevelFilter;
use sqlx::{
    ConnectOptions, Executor,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use sqlx::PgPool;

/// Tunable connection pool parameters.
/// Use `PoolConfig::default()` for sensible production settings,
/// or set fields individually for testing/low-traffic environments.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Hard upper bound on simultaneous Postgres connections. Default: 50.
    pub max_connections: u32,
    /// Connections kept warm at startup. Default: 5.
    pub min_connections: u32,
    /// Time to wait for a spare connection before returning an error. Default: 15s.
    pub acquire_timeout: Duration,
    /// Idle connections are closed after this duration. Default: 600s.
    pub idle_timeout: Duration,
    /// Connections older than this are recycled regardless of idle state. Default: 1800s.
    pub max_lifetime: Duration,
    /// Prepared statement LRU cache per connection. Default: 256.
    pub statement_cache_capacity: usize,
    /// Queries slower than this threshold are logged at WARN level. Default: 250ms.
    pub slow_statement_threshold: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 50,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(15),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            statement_cache_capacity: 256,
            slow_statement_threshold: Duration::from_millis(250),
        }
    }
}

/// Create a pool with default settings.
pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    create_pool_with_config(database_url, &PoolConfig::default()).await
}

/// Create a pool with explicit tuning parameters.
pub async fn create_pool_with_config(
    database_url: &str,
    config: &PoolConfig,
) -> Result<PgPool, AppError> {
    let connect_options = PgConnectOptions::from_str(database_url)
        .map_err(|e| AppError::Database(e.into()))?
        .application_name("cms-server")
        .statement_cache_capacity(config.statement_cache_capacity)
        .log_slow_statements(LevelFilter::Warn, config.slow_statement_threshold);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(Some(config.idle_timeout))
        .max_lifetime(Some(config.max_lifetime))
        // Reset all app.* GUCs when a connection is returned to the pool.
        // Without this, workspace context set in one request is inherited by
        // the next request that reuses the connection.
        // Use raw_sql (simple protocol) — sqlx::query uses the extended/prepared
        // protocol which Postgres rejects for multi-statement strings.
        .after_release(|conn, _meta| {
            Box::pin(async move {
                conn.execute(sqlx::raw_sql(
                    "RESET app.workspace_id; \
                     RESET app.user_id; \
                     RESET app.ip_address; \
                     RESET app.user_agent; \
                     RESET app.session_id; \
                     RESET app.correlation_id",
                ))
                .await?;
                Ok(true)
            })
        })
        .connect_with(connect_options)
        .await
        .map_err(|e| {
            tracing::error!("Database connection failed: {}", e);
            AppError::DatabaseConnectionFailed
        })?;

    tracing::info!(
        max_connections = config.max_connections,
        min_connections = config.min_connections,
        idle_timeout_secs = config.idle_timeout.as_secs(),
        max_lifetime_secs = config.max_lifetime.as_secs(),
        statement_cache_capacity = config.statement_cache_capacity,
        slow_statement_ms = config.slow_statement_threshold.as_millis(),
        "PostgreSQL connection pool established"
    );

    Ok(pool)
}

/// Run a lightweight `SELECT 1` to verify the pool is healthy.
pub async fn health_check(pool: &PgPool) -> Result<bool, AppError> {
    let (val,): (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))?;
    Ok(val == 1)
}
