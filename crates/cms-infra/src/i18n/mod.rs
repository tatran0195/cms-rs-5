//! Generic, dependency-light i18n core: [`Locale`] → [`Catalog`] → formatted text.

pub mod catalog;
pub mod context;
pub mod errors;
pub mod locale;
pub mod message;

pub use catalog::{Audit, Catalog, CatalogBuilder, CatalogError, I18nError};
pub use context::{current_locale, with_locale};
pub use errors::{embedded_catalog, error_code_to_key, localize_error};
pub use locale::{accept_language_tags, Direction, Locale, LocaleError};
pub use message::{Args, Message, ParseError};

/// Localize an AppError code and details using the task-local locale and embedded catalog
pub fn localize_app_error(error_code: &str, details_arg: Option<&str>) -> cms_error::LocalizedErrorInfo {
    let req_locale = current_locale();
    let served_locale = embedded_catalog().negotiate([req_locale.as_str()]);
    let mut args = Args::new();
    if let Some(d) = details_arg {
        args.set("details", d);
        args.set("path", d);
    }
    let key = error_code_to_key(error_code);
    let message = localize_error(error_code, &req_locale, &args);
    cms_error::LocalizedErrorInfo {
        key,
        message,
        locale_code: served_locale.as_str().to_string(),
    }
}

/// Register this i18n implementation as the global error localizer in `cms-error`
pub fn init_error_localizer() {
    cms_error::set_error_localizer(localize_app_error);
}
