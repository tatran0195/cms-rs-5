//! CMS Model Context Protocol (MCP) Server.
//!
//! This crate provides the official Model Context Protocol (MCP) server implementation
//! powered by `rmcp`. It allows AI agents (e.g. Claude Desktop, Cursor, Antigravity) to query
//! and search CMS documentation programmatically via Streamable HTTP (Axum) or stdio transport.

pub mod handler;
pub mod services;
pub mod types;

use std::sync::Arc;

pub use handler::*;
use rmcp::{
    service::ServiceExt,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, session::local::LocalSessionManager,
        tower::StreamableHttpService,
    },
};
pub use services::*;
pub use types::*;

/// Construct a Streamable HTTP Tower service for mounting into Axum routers.
pub fn create_mcp_http_service(
    ctx: Arc<McpContext>,
    security: McpSecurityContext,
) -> StreamableHttpService<CmsMcpHandler, LocalSessionManager> {
    let handler = CmsMcpHandler::new(ctx, security);
    StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().with_json_response(true),
    )
}

/// Run the MCP server over stdio transport.
pub async fn run_stdio(
    ctx: Arc<McpContext>,
    security: McpSecurityContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let handler = CmsMcpHandler::new(ctx, security);
    let running = handler.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(())
}

/// Handler for listing MCP audit events
pub async fn list_mcp_audit_events_handler(
    axum::extract::State(state): axum::extract::State<Arc<cms_server_core::app_state::AppState>>,
    auth: cms_auth::AuthExtractor,
    axum::extract::Query(query): axum::extract::Query<cms_entity::mcp::ListMcpAuditEventsQuery>,
) -> Result<axum::Json<cms_entity::common::PaginatedResponse<cms_entity::mcp::McpAuditEventResponse>>, cms_error::AppError> {
    let result = McpService::list_audit_events(
        state.db(),
        &auth.user.id,
        query.project_id.as_deref(),
        query.user_id.as_deref(),
        query.operation.as_deref(),
        query.limit,
        query.offset,
    )
    .await?;

    Ok(axum::Json(result))
}

/// Mount the MCP subrouter including audit events and streamable HTTP fallback service
pub fn router(state: Arc<cms_server_core::app_state::AppState>) -> axum::Router {
    let mcp_service = create_mcp_http_service(
        Arc::new(McpContext::new(
            state.db().clone(),
            Arc::clone(state.authz()),
        )),
        McpSecurityContext::system(),
    );

    axum::Router::new()
        .route("/audit-events", axum::routing::get(list_mcp_audit_events_handler))
        .with_state(state)
        .fallback_service(mcp_service)
}
