//! OpenAPI Business Logic Service
//!
//! Provides document management, SSRF-protected remote fetching, and parsing.

use chrono::Utc;
use cms_db::{openapi::OpenApiDocumentQueries, PgPool};
use cms_entity::openapi::{
    CreateOpenApiDocumentRequest, OpenApiDocumentResponse, OpenApiParsingResult,
    ParseOpenApiDocumentRequest, UpdateOpenApiDocumentRequest,
};
use cms_error::AppError;

/// OpenAPI service decoupled to accept `&PgPool` directly.
#[derive(Debug, Clone, Copy)]
pub struct OpenApiService;

impl OpenApiService {
    /// Create a new OpenAPI document
    pub async fn create_document(
        pool: &PgPool,
        _user_id: &str,
        request: CreateOpenApiDocumentRequest,
    ) -> Result<OpenApiDocumentResponse, AppError> {
        let existing =
            OpenApiDocumentQueries::get_by_url(pool, &request.url, &request.project_id)
                .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(
                "An OpenAPI document with this URL already exists for this project".to_string(),
            ));
        }

        let document = OpenApiDocumentQueries::create(
            pool,
            &request.project_id,
            &request.name,
            &request.url,
        )
        .await?;

        Ok(document.into())
    }

    /// Get OpenAPI document by ID
    pub async fn get_document(
        pool: &PgPool,
        _user_id: &str,
        document_id: &str,
    ) -> Result<OpenApiDocumentResponse, AppError> {
        let document = OpenApiDocumentQueries::get_by_id(pool, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("OpenAPI document not found".to_string()))?;

        Ok(document.into())
    }

    /// List OpenAPI documents for a project
    pub async fn list_documents(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
    ) -> Result<Vec<OpenApiDocumentResponse>, AppError> {
        let documents = OpenApiDocumentQueries::get_by_project(pool, project_id).await?;

        Ok(documents.into_iter().map(|d| d.into()).collect())
    }

    /// Update an OpenAPI document
    pub async fn update_document(
        pool: &PgPool,
        _user_id: &str,
        document_id: &str,
        request: UpdateOpenApiDocumentRequest,
    ) -> Result<OpenApiDocumentResponse, AppError> {
        let _document = OpenApiDocumentQueries::get_by_id(pool, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("OpenAPI document not found".to_string()))?;

        let updated = OpenApiDocumentQueries::update(
            pool,
            document_id,
            request.name.as_deref(),
            request.url.as_deref(),
        )
        .await?;

        Ok(updated.into())
    }

    /// Delete an OpenAPI document
    pub async fn delete_document(
        pool: &PgPool,
        _user_id: &str,
        document_id: &str,
    ) -> Result<bool, AppError> {
        let _document = OpenApiDocumentQueries::get_by_id(pool, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("OpenAPI document not found".to_string()))?;

        OpenApiDocumentQueries::delete(pool, document_id).await
    }

    /// Parse an OpenAPI document (fetch and parse the content)
    pub async fn parse_document(
        pool: &PgPool,
        _user_id: &str,
        request: ParseOpenApiDocumentRequest,
    ) -> Result<OpenApiParsingResult, AppError> {
        let document = OpenApiDocumentQueries::get_by_id(pool, &request.id)
            .await?
            .ok_or_else(|| AppError::NotFound("OpenAPI document not found".to_string()))?;

        // Fetch the OpenAPI document from the URL
        let content = match fetch_openapi_content(&document.url).await {
            Ok(content) => content,
            Err(e) => {
                let error_msg = format!("Failed to fetch OpenAPI document: {}", e);
                OpenApiDocumentQueries::update_error(pool, &request.id, Some(&error_msg))
                    .await?;
                return Ok(OpenApiParsingResult {
                    document_id: request.id,
                    parsed_successfully: false,
                    paths_count: None,
                    error_message: Some(error_msg),
                });
            }
        };

        // Parse the OpenAPI content (JSON or YAML) and count paths
        let paths_count = count_openapi_paths(&content);

        // Update the document with parsed content
        OpenApiDocumentQueries::update_parsed(
            pool,
            &request.id,
            Some(&content),
            Some(Utc::now()),
            None,
        )
        .await?;

        Ok(OpenApiParsingResult {
            document_id: request.id,
            parsed_successfully: true,
            paths_count: Some(paths_count),
            error_message: None,
        })
    }

    /// Get OpenAPI document with paths
    pub async fn get_document_with_paths(
        pool: &PgPool,
        _user_id: &str,
        document_id: &str,
    ) -> Result<OpenApiDocumentResponse, AppError> {
        let document = OpenApiDocumentQueries::get_by_id(pool, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("OpenAPI document not found".to_string()))?;

        Ok(document.into())
    }

    /// Get OpenAPI document content
    pub async fn get_document_content(
        pool: &PgPool,
        _user_id: &str,
        document_id: &str,
    ) -> Result<serde_json::Value, AppError> {
        let document = OpenApiDocumentQueries::get_by_id(pool, document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("OpenAPI document not found".to_string()))?;

        if let Some(content_str) = document.content {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content_str) {
                return Ok(json_val);
            }
        }

        // Return empty valid spec structure if not yet parsed
        Ok(
            serde_json::json!({ "openapi": "3.0.0", "info": { "title": document.name, "version": "1.0.0" }, "paths": {} }),
        )
    }
}

pub(crate) fn is_private_or_restricted_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            ipv4.is_loopback()
                || ipv4.is_private()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_multicast()
                || ipv4.octets()[0] == 0 // 0.0.0.0/8 (Current network)
                || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0xc0) == 64) // CGNAT 100.64.0.0/10
                || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 0) // 192.0.0.0/24 (IETF Protocol)
                || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2) // 192.0.2.0/24 (TEST-NET-1)
                || (ipv4.octets()[0] == 198 && (ipv4.octets()[1] == 18 || ipv4.octets()[1] == 19)) // 198.18.0.0/15 (Benchmarking)
                || (ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100) // 198.51.100.0/24 (TEST-NET-2)
                || (ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113) // 203.0.113.0/24 (TEST-NET-3)
                || ipv4.octets()[0] >= 240 // 240.0.0.0/4 (Reserved / Future use)
        }
        std::net::IpAddr::V6(ipv6) => {
            ipv6.is_loopback()
                || ipv6.is_multicast()
                || ipv6.is_unspecified()
                || ((ipv6.segments()[0] & 0xfe00) == 0xfc00) // Unique local (fc00::/7)
                || ((ipv6.segments()[0] & 0xffc0) == 0xfe80) // Link-local (fe80::/10)
                || (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0xdb8) // 2001:db8::/32 (Documentation)
                || (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0x2 && ipv6.segments()[2] == 0) // 2001:2::/48 (Benchmarking)
                || (ipv6.segments()[0] == 0x0100) // 100::/64 (Discard prefix)
                || ipv6.to_ipv4_mapped().map(|v4| is_private_or_restricted_ip(std::net::IpAddr::V4(v4))).unwrap_or(false)
                || ipv6.to_ipv4().map(|v4| is_private_or_restricted_ip(std::net::IpAddr::V4(v4))).unwrap_or(false)
        }
    }
}

/// Fetch OpenAPI content from a URL via HTTP with SSRF protection, DNS pinning, and bounded streaming
async fn fetch_openapi_content(url_str: &str) -> Result<String, String> {
    let parsed_url = reqwest::Url::parse(url_str).map_err(|e| format!("Invalid URL: {e}"))?;

    let scheme = parsed_url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("Only HTTP and HTTPS URLs are allowed".to_string());
    }

    let host = parsed_url
        .host_str()
        .ok_or_else(|| "URL has no valid host".to_string())?;

    let port = parsed_url.port_or_known_default().unwrap_or(80);

    // Resolve domain and check for private / restricted IPs
    let addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| format!("DNS resolution failed for '{host}': {e}"))?;

    let mut valid_ip = None;
    let mut resolved_any = false;
    for addr in addrs {
        resolved_any = true;
        let ip = addr.ip();
        if is_private_or_restricted_ip(ip) {
            return Err(format!(
                "Access to private or restricted IP ({ip}) is forbidden"
            ));
        }
        if valid_ip.is_none() {
            valid_ip = Some(ip);
        }
    }

    if !resolved_any {
        return Err(format!(
            "Host '{host}' could not be resolved to any IP address"
        ));
    }

    let valid_ip = valid_ip.ok_or_else(|| "No valid public IP found".to_string())?;

    // PIN DNS resolution to the exact validated socket address to prevent DNS rebinding TOCTOU attacks
    let target_addr = std::net::SocketAddr::new(valid_ip, port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none()) // Prevent redirect-based SSRF bypass
        .resolve(host, target_addr)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let mut response = client
        .get(parsed_url)
        .header("User-Agent", "cms-CMS/1.0")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error status: {}", response.status()));
    }

    const MAX_SIZE: usize = 5 * 1024 * 1024; // 5MB
    let mut total_bytes = 0usize;
    let mut body_bytes = Vec::new();

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Failed to stream response chunk: {e}"))?
    {
        total_bytes += chunk.len();
        if total_bytes > MAX_SIZE {
            return Err("OpenAPI document exceeds maximum size limit (5MB)".to_string());
        }
        body_bytes.extend_from_slice(&chunk);
    }

    String::from_utf8(body_bytes).map_err(|e| format!("OpenAPI document is not valid UTF-8: {e}"))
}

/// Parse OpenAPI content to count endpoints defined in `paths` (supports both JSON and YAML)
fn count_openapi_paths(content: &str) -> i32 {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(paths) = val.get("paths").and_then(|p| p.as_object()) {
            return paths.len() as i32;
        }
    }

    if let Ok(docs) = yaml_rust2::YamlLoader::load_from_str(content) {
        if let Some(doc) = docs.first() {
            if let Some(paths) = doc["paths"].as_hash() {
                return paths.len() as i32;
            }
        }
    }

    0
}
