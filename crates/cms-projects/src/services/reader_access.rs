//! Project reader access business logic decoupled from `BizContext`.

use chrono::{Duration, Utc};
use cms_entity::reader_access::{
    Audience, JwtAccessProvider, ProjectAudienceItem, ProjectJwtProviderItem,
    ProjectReaderAccessResponse, ProjectReaderEmergencyRevokeResponse,
    ProjectReaderInvitationResponse, ProjectReaderItem, Reader,
};
use cms_error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::queries::reader_access::{
    AudienceQueries, JwtAccessProviderQueries, ReaderAudienceQueries, ReaderInvitationQueries,
    ReaderQueries,
};

/// Reader access service
#[derive(Debug)]
pub struct ReaderAccessService;

impl ReaderAccessService {
    /// Map a stored Reader/Audience pair into the SPA `ReaderAccessData` shape.
    #[must_use]
    pub fn reader_access_to_spa(
        r_access_mode: &str,
        readers: &[Reader],
        audiences: &[Audience],
        provider: &Option<JwtAccessProvider>,
        audit: &[serde_json::Value],
    ) -> ProjectReaderAccessResponse {
        let readers_items: Vec<ProjectReaderItem> = readers
            .iter()
            .map(|r| ProjectReaderItem {
                id: r.id.clone(),
                email: r.email.clone(),
                name: r.name.clone(),
                status: "active".to_string(),
                audiences: vec![],
                count: serde_json::json!({ "sessions": 0 }),
            })
            .collect();

        let audiences_items: Vec<ProjectAudienceItem> =
            audiences.iter().map(Self::audience_to_spa).collect();

        let jwt_item = provider.as_ref().map(|p| ProjectJwtProviderItem {
            enabled: true,
            issuer: p.issuer.clone(),
            audience: p.audience.clone(),
            jwks_url: None,
            public_jwks: None,
            groups_claim: "groups".to_string(),
            claim_mapping: serde_json::json!({}),
            session_ttl_minutes: 60,
            max_token_age_seconds: 86400,
            clock_tolerance_secs: 60,
        });

        ProjectReaderAccessResponse {
            access_mode: r_access_mode.to_string(),
            readers: readers_items,
            audiences: audiences_items,
            jwt: jwt_item,
            audit: audit.to_vec(),
        }
    }

    #[must_use]
    pub fn audience_to_spa(a: &Audience) -> ProjectAudienceItem {
        ProjectAudienceItem {
            id: a.id.clone(),
            name: a.name.clone(),
            grants: vec![],
            count: serde_json::json!({ "readers": 0 }),
        }
    }

    /// Get reader access configuration for a project
    pub async fn get_reader_access(
        pool: &PgPool,
        project_id: &str,
    ) -> Result<ProjectReaderAccessResponse, AppError> {
        let audiences = AudienceQueries::get_by_project(pool, project_id).await?;
        let readers = ReaderQueries::get_by_project(pool, project_id).await?;

        let provider =
            JwtAccessProviderQueries::get_by_issuer_and_audience(pool, project_id, project_id)
                .await
                .ok()
                .flatten();

        let access_mode = if provider.is_some() || !readers.is_empty() {
            "READERS"
        } else if audiences.is_empty() {
            "PUBLIC"
        } else {
            "READERS"
        };

        Ok(Self::reader_access_to_spa(
            access_mode,
            &readers,
            &audiences,
            &provider,
            &[],
        ))
    }

    /// Create a reader audience
    pub async fn create_audience(
        pool: &PgPool,
        project_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<ProjectAudienceItem, AppError> {
        let audience = AudienceQueries::create(pool, project_id, name, description).await?;
        Ok(Self::audience_to_spa(&audience))
    }

    /// Delete a reader audience
    pub async fn delete_audience(pool: &PgPool, audience_id: &str) -> Result<bool, AppError> {
        AudienceQueries::delete(pool, audience_id).await
    }

    /// Invite a reader
    pub async fn invite_reader(
        pool: &PgPool,
        audience_id: &str,
        email: &str,
    ) -> Result<ProjectReaderInvitationResponse, AppError> {
        let token = format!("nblrinv_{}", Uuid::new_v4().simple());
        let expires_at = Utc::now() + Duration::days(7);

        let invitation =
            ReaderInvitationQueries::create(pool, audience_id, email, &token, expires_at).await?;

        Ok(ProjectReaderInvitationResponse {
            id: invitation.id,
            email: invitation.email,
            audience_id: invitation.audience_id,
            token: invitation.token,
            expires_at: invitation.expires_at.to_rfc3339(),
            created_at: invitation.created_at.to_rfc3339(),
        })
    }

    /// Revoke a reader from an audience
    pub async fn revoke_reader(
        pool: &PgPool,
        reader_id: &str,
        audience_id: &str,
    ) -> Result<bool, AppError> {
        ReaderAudienceQueries::delete(pool, reader_id, audience_id).await
    }

    /// Configure reader JWT access provider
    pub async fn configure_jwt(
        pool: &PgPool,
        name: &str,
        issuer: &str,
        audience: &str,
        secret: &str,
    ) -> Result<ProjectJwtProviderItem, AppError> {
        let provider =
            JwtAccessProviderQueries::create(pool, name, issuer, audience, secret).await?;

        Ok(ProjectJwtProviderItem {
            enabled: true,
            issuer: provider.issuer,
            audience: provider.audience,
            jwks_url: None,
            public_jwks: None,
            groups_claim: "groups".to_string(),
            claim_mapping: serde_json::json!({}),
            session_ttl_minutes: 60,
            max_token_age_seconds: 86400,
            clock_tolerance_secs: 60,
        })
    }

    /// Emergency revoke JWT provider for project
    pub async fn emergency_revoke(
        pool: &PgPool,
        project_id: &str,
    ) -> Result<ProjectReaderEmergencyRevokeResponse, AppError> {
        let provider =
            JwtAccessProviderQueries::get_by_issuer_and_audience(pool, project_id, project_id)
                .await?;

        let revoked = if let Some(p) = provider {
            JwtAccessProviderQueries::delete(pool, &p.id)
                .await
                .unwrap_or(false)
        } else {
            false
        };

        Ok(ProjectReaderEmergencyRevokeResponse {
            revoked,
            success: true,
        })
    }
}
