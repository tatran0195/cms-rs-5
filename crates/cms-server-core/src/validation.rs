//! Input validation rules and per-field error collection.
//!
//! Provides reusable validators preventing stored XSS, weak passwords,
//! invalid codes, and malformed inputs.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Collects per-field validation errors.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ValidationErrors {
    fields: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        self.fields
            .entry(field.to_owned())
            .or_default()
            .push(message.into());
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.fields.is_empty()
    }

    #[must_use]
    pub fn into_fields(self) -> HashMap<String, Vec<String>> {
        self.fields
    }
}

// ── Compiled Regex Patterns ────────────────────────────────────────────────

static CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Z0-9][A-Z0-9\-]*[A-Z0-9]$").unwrap_or_else(|_| unreachable!())
});

static SLUG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9][a-z0-9\-]*[a-z0-9]$").unwrap_or_else(|_| unreachable!())
});

static USERNAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap_or_else(|_| unreachable!()));

static EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$")
        .unwrap_or_else(|_| unreachable!())
});

static HEX_COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap_or_else(|_| unreachable!()));

static HTML_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap_or_else(|_| unreachable!()));

static API_ENDPOINT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/api/[a-z0-9/\-_]+$").unwrap_or_else(|_| unreachable!()));

// ── Field Validators ───────────────────────────────────────────────────────

pub fn validate_code(errors: &mut ValidationErrors, field: &str, value: &str) {
    if value.len() < 2 {
        errors.add(field, "Code must be at least 2 characters");
    } else if value.len() > 20 {
        errors.add(field, "Code must be at most 20 characters");
    } else if !CODE_RE.is_match(value) {
        errors.add(
            field,
            "Code must be uppercase alphanumeric with optional hyphens, no leading/trailing hyphen",
        );
    }
}

pub fn validate_slug(errors: &mut ValidationErrors, field: &str, value: &str) {
    if value.len() < 2 {
        errors.add(field, "Slug must be at least 2 characters");
    } else if value.len() > 100 {
        errors.add(field, "Slug must be at most 100 characters");
    } else if !SLUG_RE.is_match(value) {
        errors.add(
            field,
            "Slug must be lowercase alphanumeric with hyphens, no leading/trailing hyphen",
        );
    }
}

pub fn validate_name(errors: &mut ValidationErrors, field: &str, value: &str) {
    let trimmed = value.trim();
    if trimmed.len() < 2 {
        errors.add(field, "Name must be at least 2 characters");
    } else if trimmed.chars().count() > 100 {
        errors.add(field, "Name must be at most 100 characters");
    }
}

pub fn validate_username(errors: &mut ValidationErrors, field: &str, value: &str) {
    if value.len() < 3 {
        errors.add(field, "Username must be at least 3 characters");
    } else if value.len() > 30 {
        errors.add(field, "Username must be at most 30 characters");
    } else if !USERNAME_RE.is_match(value) {
        errors.add(
            field,
            "Username must start with a letter and contain only lowercase letters, digits, and underscores",
        );
    }
}

pub fn validate_email(errors: &mut ValidationErrors, field: &str, value: &str) {
    if !EMAIL_RE.is_match(value) {
        errors.add(field, "Invalid email address");
    }
}

pub fn validate_optional_email(errors: &mut ValidationErrors, field: &str, value: &str) {
    if !value.is_empty() && !EMAIL_RE.is_match(value) {
        errors.add(field, "Invalid email address");
    }
}

pub fn validate_password(errors: &mut ValidationErrors, field: &str, value: &str) {
    if value.len() < 8 {
        errors.add(field, "Password must be at least 8 characters");
    }
    if !value.chars().any(char::is_uppercase) {
        errors.add(field, "Password must contain at least one uppercase letter");
    }
    if !value.chars().any(char::is_lowercase) {
        errors.add(field, "Password must contain at least one lowercase letter");
    }
    if !value.chars().any(|c| c.is_ascii_digit()) {
        errors.add(field, "Password must contain at least one digit");
    }
    if !value.chars().any(|c| !c.is_alphanumeric()) {
        errors.add(
            field,
            "Password must contain at least one special character",
        );
    }
}

pub fn validate_optional_url(errors: &mut ValidationErrors, field: &str, value: &str) {
    if !value.is_empty() && url::Url::parse(value).is_err() {
        errors.add(field, "Invalid URL format");
    }
}

pub fn validate_hex_color(errors: &mut ValidationErrors, field: &str, value: &str) {
    if !HEX_COLOR_RE.is_match(value) {
        errors.add(field, "Must be a valid hex color (#RRGGBB)");
    }
}

/// Reject strings containing HTML tags to prevent stored XSS attacks.
pub fn validate_no_html(errors: &mut ValidationErrors, field: &str, value: &str) {
    if HTML_TAG_RE.is_match(value) {
        errors.add(field, "HTML tags are not allowed");
    }
}

/// Validate that an internal endpoint starts with `/api/`.
pub fn validate_api_endpoint(errors: &mut ValidationErrors, field: &str, value: &str) {
    if !API_ENDPOINT_RE.is_match(value) {
        errors.add(
            field,
            "Endpoint must start with /api/ and contain only lowercase letters, digits, hyphens, underscores, and slashes",
        );
    }
}

// ── Axum Validation Extractors ───────────────────────────────────────────────

use axum::{
    extract::{FromRequest, FromRequestParts, Path, Query, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
    Json,
};
use cms_error::AppError;
use serde::de::DeserializeOwned;
use validator::Validate;

/// Validated JSON extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid JSON: {}", e)))?;

        validate(&value)?;
        Ok(Self(value))
    }
}

impl<T> IntoResponse for ValidatedJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

impl<T> std::ops::Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Validated query extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid query parameters: {}", e)))?;

        validate(&value)?;
        Ok(Self(value))
    }
}

impl<T> std::ops::Deref for ValidatedQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Validated path extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedPath<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedPath<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid path parameters: {}", e)))?;

        validate(&value)?;
        Ok(Self(value))
    }
}

impl<T> std::ops::Deref for ValidatedPath<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Validate a value using `validator::Validate` and return an `AppError::Validation` if invalid
pub fn validate<T: Validate>(value: &T) -> Result<(), AppError> {
    value.validate().map_err(|e| {
        let errors: Vec<String> = e
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| e.code.to_string())
                    })
                    .collect();
                format!("{}:", field) + &error_messages.join(", ")
            })
            .collect();
        AppError::Validation(errors.join("; "))
    })
}
