//! Analytics domain business service.

use cms_db::{
    analytics::{AnalyticsEventQueries, AnalyticsQueries},
    page::PageQueries,
    project::ProjectQueries,
    PgPool,
};
use cms_entity::{
    analytics::{
        AnalyticsDashboardResponse, AnalyticsEventResponse, AnalyticsQueryRequest,
        AnalyticsQueryResponse, ListAnalyticsEventsQuery, TrackAnalyticsEventRequest,
    },
    common::PaginatedResponse,
};
use cms_error::AppError;

/// Analytics service decoupled from BizContext.
#[derive(Debug, Clone, Copy)]
pub struct AnalyticsService;

impl AnalyticsService {
    /// Record an analytics event
    #[allow(clippy::too_many_arguments)]
    pub async fn record_event(
        pool: &PgPool,
        project_id: Option<&str>,
        user_id: Option<&str>,
        event_type: &str,
        metadata: serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AnalyticsEventResponse, AppError> {
        let event = AnalyticsEventQueries::create(
            pool, project_id, user_id, event_type, metadata, ip_address, user_agent,
        )
        .await?;

        Ok(event.into())
    }

    /// Query analytics events
    pub async fn query_events(
        pool: &PgPool,
        _user_id: &str,
        request: AnalyticsQueryRequest,
        page: u64,
        page_size: u64,
    ) -> Result<AnalyticsQueryResponse, AppError> {
        if let Some(ref proj_id) = request.project_id {
            let _project = ProjectQueries::get_by_id(pool, proj_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Project {proj_id} not found")))?;
        }

        let limit = page_size as i64;
        let offset = ((page.saturating_sub(1)) as i64) * limit;

        let events = AnalyticsEventQueries::query(
            pool,
            request.project_id.as_deref(),
            request.user_id.as_deref(),
            request.event_type.as_deref(),
            request.start_date,
            request.end_date,
            limit,
            offset,
        )
        .await?;

        let total = AnalyticsEventQueries::count(
            pool,
            request.project_id.as_deref(),
            request.user_id.as_deref(),
            request.event_type.as_deref(),
            request.start_date,
            request.end_date,
        )
        .await?;

        let event_responses: Vec<AnalyticsEventResponse> =
            events.into_iter().map(Into::into).collect();

        Ok(AnalyticsQueryResponse {
            query: request,
            results: vec![],
            total,
            events: event_responses,
            page: page as i64,
            page_size: page_size as i64,
        })
    }

    /// Get analytics summary
    pub async fn get_summary(
        pool: &PgPool,
        _user_id: &str,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value, AppError> {
        AnalyticsQueries::get_summary(pool, start_date, end_date).await
    }

    /// Track an analytics event
    pub async fn track_event(
        pool: &PgPool,
        request: TrackAnalyticsEventRequest,
    ) -> Result<AnalyticsEventResponse, AppError> {
        Self::record_event(
            pool,
            request.project_id.as_deref(),
            request.user_id.as_deref(),
            &request.event_type,
            request.metadata,
            request.ip_address.as_deref(),
            request.user_agent.as_deref(),
        )
        .await
    }

    /// List analytics events
    pub async fn list_events(
        pool: &PgPool,
        user_id: &str,
        query: ListAnalyticsEventsQuery,
    ) -> Result<PaginatedResponse<AnalyticsEventResponse>, AppError> {
        let page = query.page.unwrap_or(1) as u64;
        let page_size = query.page_size.unwrap_or(20) as u64;
        let response = Self::query_events(pool, user_id, query, page, page_size).await?;
        Ok(PaginatedResponse::new(
            response.events,
            response.total as u64,
            page,
            page_size,
        ))
    }

    /// Query analytics
    pub async fn query_analytics(
        pool: &PgPool,
        user_id: &str,
        request: AnalyticsQueryRequest,
    ) -> Result<AnalyticsQueryResponse, AppError> {
        let page = request.page.unwrap_or(1) as u64;
        let page_size = request.page_size.unwrap_or(20) as u64;
        Self::query_events(pool, user_id, request, page, page_size).await
    }

    /// Get dashboard for a project
    pub async fn get_dashboard(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<AnalyticsDashboardResponse, AppError> {
        AnalyticsQueries::get_dashboard(pool, project_id).await
    }

    /// Get page views scoped by project and authorized by user
    pub async fn get_page_views(
        pool: &PgPool,
        _user_id: &str,
        page_id: &str,
    ) -> Result<serde_json::Value, AppError> {
        let page = PageQueries::get_by_id(pool, page_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Page {page_id} not found")))?;

        AnalyticsQueries::get_page_views(pool, &page.project_id, page_id).await
    }

    /// Get system stats
    pub async fn get_system_stats(pool: &PgPool) -> Result<serde_json::Value, AppError> {
        AnalyticsQueries::get_system_stats(pool).await
    }
}

/// Process analytics job (for worker)
pub async fn process_analytics_job(
    pool: &PgPool,
    payload: &serde_json::Value,
) -> Result<(), AppError> {
    let event_type = payload
        .get("event_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing event_type".to_string()))?;
    let project_id = payload.get("project_id").and_then(|v| v.as_str());
    let user_id = payload.get("user_id").and_then(|v| v.as_str());
    let metadata = payload.get("metadata").cloned().unwrap_or_default();
    let ip_address = payload.get("ip_address").and_then(|v| v.as_str());
    let user_agent = payload.get("user_agent").and_then(|v| v.as_str());

    AnalyticsEventQueries::create(
        pool, project_id, user_id, event_type, metadata, ip_address, user_agent,
    )
    .await?;

    Ok(())
}
