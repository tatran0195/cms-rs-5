//! Domain services for content vertical slice.

pub mod branch;
pub mod language;
pub mod page;

pub use branch::BranchService;
pub use language::LanguageService;
pub use page::PageService;
