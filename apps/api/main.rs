//! CMS Server - Composition Root
//!
//! This is the main entry point for the cms-server binary.
//! It composes all the crates together and starts the Axum HTTP server.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{Router, http::StatusCode};
use cms_error::AppError;
use cms_infra::config::Config;
use cms_server::api_router;
use cms_server_core::{
    app_state::AppState,
    middleware::observability::{init_observability, metrics_handler},
};
use cms_sites::sites_router;
use cms_worker::app_state::WorkerState;
use tokio::net::TcpListener;
use tracing::info;

async fn shutdown_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("Failed to install CTRL+C signal handler: {e}");
        return;
    }
    tracing::info!("Received graceful shutdown signal. Starting shutdown...");
}

/// Build a CORS layer from the configured admin/app origin allow-list.
///
/// Requests from the SPA itself are same-origin and need no CORS. This layer
/// enables credentialed cross-origin calls (e.g. an admin console hosted on a
/// different origin) only for origins listed in `[admin_origin]`. Localhost
/// origins are also permitted in development when `allow_localhost` is set.
fn build_cors(config: &Config) -> tower_http::cors::CorsLayer {
    use axum::http::{HeaderName, HeaderValue, Method, header};
    use tower_http::cors::{AllowOrigin, CorsLayer};

    let allow_localhost = config.admin_origin.allow_localhost;
    let allowed = config.admin_origin.allowed_origins.clone();

    let allow_origin = AllowOrigin::predicate(move |origin: &HeaderValue, _| {
        let Some(origin) = origin.to_str().ok() else {
            return false;
        };
        if allowed.iter().any(|a| a == origin) {
            return true;
        }
        if allow_localhost {
            let is_localhost = origin.starts_with("http://localhost:")
                || origin.starts_with("http://127.0.0.1:")
                || origin.starts_with("http://[::1]:");
            if is_localhost {
                return true;
            }
        }
        false
    });

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            header::COOKIE,
            header::SET_COOKIE,
            header::CONTENT_LENGTH,
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-locale"),
        ])
        .max_age(Duration::from_secs(86400))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize observability (logging, tracing, metrics)
    init_observability();

    info!("Starting CMS Server...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Startup check: verify frontend assets directory
    let asset_root = cms_sites::spa::get_asset_root();
    if config.is_production() {
        cms_sites::validate_frontend_assets(&asset_root).map_err(|e| {
            AppError::Validation(format!("Frontend assets startup check failed: {e}"))
        })?;
        info!("Frontend assets verified at {}", asset_root.display());
    } else {
        match cms_sites::validate_frontend_assets(&asset_root) {
            Ok(canonical) => info!("Frontend assets verified at {}", canonical.display()),
            Err(err) => {
                tracing::warn!(
                    "Frontend assets check: {err}. In dev mode, SPA requests will return 500 \
                     until frontend is built."
                );
            }
        }
    }

    // Build application state
    let search_engine = cms_search::create_search_engine(&config.search)?;
    let state = Arc::new(AppState::from_config(&config, search_engine).await?);
    info!("Application state initialized");

    // Run real job processors over the same durable queue used by API handlers.
    // The external cms-worker binary can be scaled horizontally against the same
    // PostgreSQL queue; FOR UPDATE SKIP LOCKED prevents duplicate claims.
    let worker_state = Arc::new(WorkerState::from_app_state(&state).await?);
    let (worker_shutdown_tx, worker_shutdown_rx) = tokio::sync::watch::channel(false);
    let worker_handles = cms_worker::start_consumers_with_shutdown(
        Arc::clone(&state.job_queue),
        worker_state,
        worker_shutdown_rx,
    )
    .await?;

    // Synchronize default recurring jobs into cms_scheduled_jobs and start distributed runner
    cms_scheduler::Scheduler::sync_registry(&state.pool, None).await?;
    let scheduler_handle = cms_scheduler::Scheduler::spawn(
        state.pool.clone(),
        Arc::clone(&state.job_queue),
        None,
    );

    // Build the full router with all middleware
    let api_router = api_router(Arc::clone(&state));
    let sites_router = sites_router(Arc::clone(&state));

    // CORS configured from the `[admin_origin]` allow-list rather than the wider
    // accepted-by-any-origin default. The SPA is served same-origin, so browser
    // requests do not need CORS, but an admin/app console on another origin must
    // still be allowed with credentials. Localhost origins are permitted in dev.
    let cors = build_cors(&config);

    let app = Router::new()
        .route("/metrics", axum::routing::get(metrics_handler))
        .nest("/api", api_router)
        .merge(sites_router)
        .layer(axum::Extension(Arc::clone(&state)))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(cors)
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ));

    // Bind to socket
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    info!("HTTP server drained; signaling background workers and scheduler to terminate...");
    let _ = worker_shutdown_tx.send(true);
    let _ = scheduler_handle.stop().await;
    for handle in worker_handles {
        let _ = handle.await;
    }
    info!("All background workers and scheduler terminated cleanly.");
    Ok(())
}
