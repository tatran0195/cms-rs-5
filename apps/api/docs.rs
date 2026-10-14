//! OpenAPI Documentation Generation & Interactive Viewers

use axum::{
    http::header,
    response::{Html, IntoResponse},
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(Debug)]
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "apiKeyAuth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
            components.add_security_scheme(
                "cookieAuth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session"))),
            );
        }
    }
}

/// OpenAPI Document Specification
#[derive(OpenApi)]
#[openapi(
    info(
        title = "CMS Documentation Platform API",
        version = "1.0.0",
        description = "REST API for CMS Modern Documentation Platform (Internal Company Scope)"
    ),
    tags(
        (name = "auth", description = "Authentication, registration, sessions, and credentials"),
        (name = "admin", description = "Platform administration, telemetry, and health diagnostics"),
        (name = "platform-events", description = "Platform audit and telemetry events"),
        (name = "public", description = "Unauthenticated public reader endpoints"),
        (name = "projects", description = "Project workspace management"),
        (name = "pages", description = "Documentation pages and tree hierarchy"),
        (name = "branches", description = "Git-like version branches"),
        (name = "languages", description = "Multi-language documentation"),
        (name = "search", description = "Hybrid BM25 and vector search")
    ),
    paths(
        cms_auth::handlers::login_handler,
        cms_auth::handlers::register_handler,
        cms_admin::handlers::get_system_stats_handler,
        cms_admin::handlers::get_system_health_handler,
        cms_admin::handlers::get_system_metrics_handler,
        cms_admin::platform_events::handlers::list_platform_events_handler,
        cms_admin::platform_events::handlers::create_platform_event_handler,
        cms_admin::platform_events::handlers::get_platform_event_handler,
        cms_public::handlers::get_public_project_handler,
        cms_public::handlers::get_public_page_handler,
    ),
    components(
        schemas(
            cms_entity::auth::LoginRequest,
            cms_entity::auth::CreateUserRequest,
            cms_entity::auth::UserResponse,
            cms_entity::project::ProjectResponse,
            cms_entity::page::PageResponse,
            cms_entity::platform_event::PlatformEvent,
            cms_entity::platform_event::PlatformEventResponse,
            cms_entity::platform_event::CreatePlatformEventRequest,
        )
    ),
    modifiers(&SecurityAddon)
)]
#[derive(Debug)]
pub struct ApiDoc;

pub async fn serve_openapi_spec() -> impl IntoResponse {
    let spec = ApiDoc::openapi().to_json().unwrap_or_default();
    ([(header::CONTENT_TYPE, "application/json")], spec)
}

pub async fn serve_openapi_yaml() -> impl IntoResponse {
    let spec = ApiDoc::openapi().to_yaml().unwrap_or_default();
    ([(header::CONTENT_TYPE, "application/yaml")], spec)
}

pub async fn serve_swagger_ui() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>CMS API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        window.onload = () => {
            window.ui = SwaggerUIBundle({
                url: '/api-docs/openapi.json',
                dom_id: '#swagger-ui',
            });
        };
    </script>
</body>
</html>"#,
    )
}

pub async fn serve_redoc() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>CMS API - ReDoc</title>
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
</head>
<body>
    <redoc spec-url='/api-docs/openapi.json'></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</body>
</html>"#,
    )
}
