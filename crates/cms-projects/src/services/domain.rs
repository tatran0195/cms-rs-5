//! Project domain service decoupled from `BizContext`.

use cms_error::AppError;
use sqlx::PgPool;

use crate::queries::domain::{DomainQueries, normalize_hostname};

use crate::models::{Domain, SpaDomainResponse};

/// Domain service
#[derive(Debug)]
pub struct DomainService;

impl DomainService {
    /// Require that a domain belongs to a project via its deployment.
    pub async fn require_domain_in_project(
        pool: &PgPool,
        project_id: &str,
        domain_id: &str,
    ) -> Result<Domain, AppError> {
        let domain = DomainQueries::get_by_id(pool, domain_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Domain not found".to_string()))?;

        let deployment_project_id = DomainQueries::get_deployment_project_id(pool, &domain.deployment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Deployment not found".to_string()))?;

        if deployment_project_id != project_id {
            return Err(AppError::Forbidden);
        }

        Ok(domain)
    }

    /// List all domains configured across deployments of a project
    pub async fn list_domains_for_project(
        pool: &PgPool,
        project_id: &str,
    ) -> Result<Vec<SpaDomainResponse>, AppError> {
        let domains = DomainQueries::get_by_project(pool, project_id).await?;
        Ok(domains.iter().map(SpaDomainResponse::from_domain).collect())
    }

    /// Add a domain to the newest deployment of a project
    pub async fn add_domain(
        pool: &PgPool,
        project_id: &str,
        hostname: &str,
    ) -> Result<SpaDomainResponse, AppError> {
        let normalized = hostname.trim().trim_end_matches('.').to_lowercase();
        if normalized.is_empty() {
            return Err(AppError::InvalidInput("domain is required".to_string()));
        }

        let deployment_id = DomainQueries::get_latest_deployment_id_for_project(pool, project_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound("No deployment exists for this project".to_string())
            })?;

        let has_primary = DomainQueries::get_primary_by_deployment(pool, &deployment_id)
            .await?
            .is_some();
        let is_primary = !has_primary;

        let domain = DomainQueries::create(pool, &deployment_id, &normalized, is_primary).await?;

        Ok(SpaDomainResponse::from_domain(&domain))
    }

    /// Delete a domain
    pub async fn delete_domain(pool: &PgPool, domain_id: &str) -> Result<bool, AppError> {
        DomainQueries::delete(pool, domain_id).await
    }

    /// Set a domain as the primary domain for its deployment
    pub async fn set_primary(
        pool: &PgPool,
        deployment_id: &str,
        domain_id: &str,
    ) -> Result<Domain, AppError> {
        DomainQueries::set_primary_for_deployment(pool, deployment_id, domain_id).await
    }

    /// Mark a domain verified
    pub async fn verify(
        pool: &PgPool,
        domain_id: &str,
        expected_token: &str,
    ) -> Result<Domain, AppError> {
        DomainQueries::verify(pool, domain_id, expected_token).await
    }

    /// Check if a domain is verified and eligible for on-demand TLS issuance (Caddy on-demand TLS)
    pub async fn is_domain_allowed_for_tls(pool: &PgPool, domain: &str) -> Result<bool, AppError> {
        let Ok(normalized) = normalize_hostname(domain) else {
            return Ok(false);
        };
        let verified = DomainQueries::get_verified_by_hostname(pool, &normalized).await?;
        Ok(verified.is_some())
    }
}
