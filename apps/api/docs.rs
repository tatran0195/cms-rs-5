//! OpenAPI Documentation Generation & Interactive Viewers

use axum::{
    http::header,
    response::{Html, IntoResponse},
};
use utoipa::OpenApi;

/// OpenAPI Document Specification
#[derive(OpenApi)]
#[openapi(
    info(
        title = "CMS Documentation Platform API",
        version = "1.0.0",
        description = "REST API for CMS Modern Documentation Platform"
    ),
    tags(
        (name = "auth", description = "Authentication and sessions"),
        (name = "projects", description = "Project workspace management"),
        (name = "pages", description = "Documentation pages and tree hierarchy"),
        (name = "branches", description = "Git-like version branches"),
        (name = "languages", description = "Multi-language documentation"),
        (name = "search", description = "Hybrid BM25 and vector search")
    )
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
