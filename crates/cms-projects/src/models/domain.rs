//! Domain models and row representations.

use chrono::{DateTime, Utc};
pub use cms_entity::domain::{
    AddProjectDomainRequest, CreateDomainRequest, DeleteDomainResponse, DnsRecord, Domain,
    DomainResponse, DomainVerificationResult, ListDomainsQuery, SpaDomainResponse,
    UpdateDomainRequest, VerifyDomainRequest,
};
use sqlx::FromRow;

/// Database representation of a domain row
#[derive(Debug, Clone, FromRow)]
pub struct DomainRow {
    pub id: String,
    pub deployment_id: String,
    pub hostname: String,
    pub is_primary: bool,
    pub ssl_certificate: Option<String>,
    pub ssl_certificate_expires_at: Option<DateTime<Utc>>,
    pub ssl_status: String,
    pub ssl_last_error: Option<String>,
    pub ssl_checked_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DomainRow> for Domain {
    fn from(row: DomainRow) -> Self {
        Self {
            id: row.id,
            deployment_id: row.deployment_id,
            hostname: row.hostname,
            is_primary: row.is_primary,
            ssl_certificate: row.ssl_certificate,
            ssl_certificate_expires_at: row.ssl_certificate_expires_at,
            ssl_status: row.ssl_status,
            ssl_last_error: row.ssl_last_error,
            ssl_checked_at: row.ssl_checked_at,
            verified_at: row.verified_at,
            verification_token: row.verification_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
