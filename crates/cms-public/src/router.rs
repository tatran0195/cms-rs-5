//! Public API router
//!
//! Routers for unauthenticated reader-facing endpoints (/public/*).

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use cms_server_core::app_state::AppState;

use crate::handlers::{
    answer_public_site_handler, authorize_custom_domain_tls_handler, get_project_sitemap_handler,
    get_public_domains_resolve_handler, get_public_git_preview_handler,
    get_public_invitation_handler, get_public_meta_handler, get_public_page_handler,
    get_public_pages_handler, get_public_project_handler, get_public_site_changelog_handler,
    get_public_site_changelog_rss_handler, get_public_site_handler,
    get_public_site_llms_full_handler, get_public_site_llms_handler,
    get_public_site_markdown_handler, get_public_site_openapi_handler,
    get_public_site_page_handler, get_public_site_robots_handler, get_public_site_sitemap_handler,
    list_public_pages_handler, post_public_marketing_events_handler,
    post_public_site_events_handler, search_public_content_handler, search_public_site_handler,
};

/// Create the public router
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/meta", get(get_public_meta_handler))
        .route(
            "/setup/status",
            get(cms_onboarding::handlers::get_setup_status_handler),
        )
        .route(
            "/setup/complete",
            post(cms_onboarding::handlers::complete_setup_handler),
        )
        .route(
            "/marketing-events",
            post(post_public_marketing_events_handler),
        )
        .route("/invitations/{id}", get(get_public_invitation_handler))
        .route("/sites/{id}", get(get_public_site_handler))
        .route("/sites/{id}/page", get(get_public_site_page_handler))
        .route(
            "/sites/{id}/changelog",
            get(get_public_site_changelog_handler),
        )
        .route("/sites/{id}/events", post(post_public_site_events_handler))
        .route("/sites/{id}/search", get(search_public_site_handler))
        .route("/sites/{id}/answer", post(answer_public_site_handler))
        .route(
            "/sites/{id}/markdown",
            get(get_public_site_markdown_handler),
        )
        .route(
            "/sites/{id}/openapi.json",
            get(get_public_site_openapi_handler),
        )
        .route(
            "/sites/{id}/changelog/rss.xml",
            get(get_public_site_changelog_rss_handler),
        )
        .route(
            "/sites/{id}/sitemap.xml",
            get(get_public_site_sitemap_handler),
        )
        .route(
            "/sites/{id}/robots.txt",
            get(get_public_site_robots_handler),
        )
        .route("/sites/{id}/llms.txt", get(get_public_site_llms_handler))
        .route(
            "/sites/{id}/llms-full.txt",
            get(get_public_site_llms_full_handler),
        )
        .route("/pages/{id}", get(get_public_pages_handler))
        .route("/git/previews/{token}", get(get_public_git_preview_handler))
        .route("/domains/resolve", get(get_public_domains_resolve_handler))
        .route(
            "/domains/tls-authorize",
            get(authorize_custom_domain_tls_handler),
        )
        .route("/projects/{project_slug}", get(get_public_project_handler))
        .route(
            "/projects/{project_slug}/pages",
            get(list_public_pages_handler),
        )
        .route(
            "/projects/{project_slug}/pages/{*page_path}",
            get(get_public_page_handler),
        )
        .route(
            "/projects/{project_slug}/search",
            get(search_public_content_handler),
        )
        .route(
            "/projects/{project_slug}/sitemap",
            get(get_project_sitemap_handler),
        )
        .route("/search/{project_slug}", get(search_public_content_handler))
        .route("/sitemap/{project_slug}", get(get_project_sitemap_handler))
        .with_state(state)
}
