use std::sync::OnceLock;

use super::{
    catalog::{Catalog, CatalogBuilder},
    locale::Locale,
    message::Args,
};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/i18n_embedded_messages.rs"));
}

static EMBEDDED_CATALOG: OnceLock<Catalog> = OnceLock::new();

pub fn embedded_catalog() -> &'static Catalog {
    EMBEDDED_CATALOG.get_or_init(|| {
        let mut builder = CatalogBuilder::new(Locale::default());
        for (locale, json) in embedded::MESSAGES {
            if let Err(err) = builder.append_json(locale, json) {
                tracing::warn!("Failed to append embedded catalog for {locale}: {err}");
            }
        }
        builder.build().unwrap_or_else(|_| {
            let mut fallback_builder = CatalogBuilder::new(Locale::default());
            let _ = fallback_builder.append_json(
                "en",
                r#"{"api_error_internal_error":"An internal error occurred"}"#,
            );
            fallback_builder
                .build()
                .unwrap_or_else(|_| Catalog::empty(Locale::default()))
        })
    })
}

/// Convert an API error code (`auth:invalid_credentials`) to an i18n key (`api_error_auth_invalid_credentials`).
pub fn error_code_to_key(code: &str) -> String {
    format!("api_error_{}", code.replace([':', '-', '.'], "_"))
}

/// Localize an error code for the given locale using the embedded catalog and variable arguments.
pub fn localize_error(code: &str, locale: &Locale, args: &Args) -> Option<String> {
    let key = error_code_to_key(code);
    let cat = embedded_catalog();
    let resolved_locale = cat.negotiate([locale.as_str()]);
    cat.format(&resolved_locale, &key, args).ok()
}
