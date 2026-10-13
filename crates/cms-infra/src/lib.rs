#![forbid(unsafe_code)]

//! Consolidated infrastructure layer for CMS.
//!
//! Provides core runtime and infrastructure services:
//! - [`config`]: Application configuration schemas & Figment loaders
//! - [`storage`]: Pluggable local and cloud (S3) object storage
//! - [`search`]: Tantivy + Lindera + FastEmbed hybrid search engine
//! - [`queue`]: In-memory and PostgreSQL task queue abstractions
//! - [`mailer`]: Lettre + MiniJinja transactional email and renderer
//! - [`analytics`]: Operational metrics and analytics event storage
//! - [`i18n`]: BCP-47 locale negotiation and string catalogs

pub mod config;
pub mod i18n;
pub mod mailer;
pub mod queue;
pub mod storage;

pub use config::Config;
pub use i18n::{Catalog, Locale};
pub use mailer::{EmailRenderer, Mailer};
pub use queue::{JobEnvelope, JobId, JobQueue, JobStatus, JobType};
pub use storage::{Storage, StorageService, StorageUrlService};

/// Initialize infrastructure runtime hooks (e.g. wire i18n error localizer into `cms-error`)
pub fn init() {
    i18n::init_error_localizer();
}
