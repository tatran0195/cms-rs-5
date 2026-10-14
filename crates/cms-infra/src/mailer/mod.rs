//! `cms-mailer` — localised transactional email: composition, templating, delivery.
//!
//! ```text
//!  packages/i18n/messages/{locale}.json   (shared with the web UI, Paraglide format)
//!        │  build.rs keeps the `email_*` keys and embeds them
//!        ▼
//!  i18n::Catalog ──audit at boot──► EmailRenderer::render(&Email, &Locale)
//!                                        │ compose: which keys + arguments
//!                                        ▼
//!                                   templates::TemplateEngine (layout only)
//!                                        ▼
//!                                   RenderedEmail ──► transport::Mailer
//! ```
//!
//! * **One source of truth** for strings: no translation lives in Rust.
//! * **Fail at boot, not at send time**: missing keys / bad placeholders abort startup.
//! * **Open locale set**: adding a language = adding `{locale}.json`, no code change.

pub mod email;
pub mod error;
pub mod keys;
pub mod render;
pub mod service;
pub mod templates;
pub mod transport;
pub mod types;

pub use service::{EmailService, process_email_job, process_email_job_with_renderer};

pub use crate::i18n;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/email_embedded_messages.rs"));
}

pub use email::Email;
pub use error::RenderError;
pub use i18n::{Catalog, Locale, accept_language_tags};
pub use render::{EmailRenderer, embedded_catalog_builder};
pub use templates::TemplateEngine;
pub use transport::{
    Mailer, MockMailer, NoopMailer, SentEmail, SmtpMailer, UnconfiguredMailer, create_mailer,
    create_mailer_with_noop_fallback,
};
pub use types::*;
