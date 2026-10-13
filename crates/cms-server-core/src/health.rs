//! Health check endpoint handler.

use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::Utc;
use cms_entity::common::SystemHealthResponse;
use cms_error::AppError;

use crate::app_state::AppState;

/// Health check handler returning system and database connectivity status.
pub async fn health_check_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SystemHealthResponse>, AppError> {
    let start = std::time::Instant::now();
    let db_status = match sqlx::query("SELECT 1").execute(state.db()).await {
        Ok(_) => "connected",
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            "disconnected"
        }
    };
    let latency_ms = start.elapsed().as_millis();
    let status = if db_status == "connected" {
        "healthy"
    } else {
        "degraded"
    };

    Ok(Json(SystemHealthResponse {
        status: status.to_string(),
        database: db_status.to_string(),
        database_latency_ms: latency_ms,
        timestamp: Utc::now().to_rfc3339(),
    }))
}
