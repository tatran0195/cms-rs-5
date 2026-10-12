//! CMS Shared Server Services Crate.
//!
//! Provides cross-domain business services and integration helpers to break
//! cyclic dependencies between leaf vertical slices:
//! - `auth_audit`: Tamper-evident audit logging and hash-chain verification
//! - `storage_url`: Presigned URL generation and asset location formatting
//! - `tenant_context`: Workspace membership validation and transaction contexts
//! - `event_dispatcher`: Atomic transactional outbox event queuing

pub mod auth_audit;
pub mod event_dispatcher;
pub mod storage_url;
pub mod tenant_context;

pub use auth_audit::AuthAuditService;
pub use event_dispatcher::EventDispatcherService;
pub use storage_url::StorageUrlService;
pub use tenant_context::TenantContextService;
