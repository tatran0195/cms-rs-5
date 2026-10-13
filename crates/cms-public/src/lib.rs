//! CMS Public vertical slice
//!
//! Encapsulates unauthenticated reader endpoints: site metadata, content resolution,
//! full-text site search, AI reader answers, changelogs, SEO artifacts (sitemap, robots, llms.txt).

pub mod handlers;
pub mod router;

pub use router::router;
