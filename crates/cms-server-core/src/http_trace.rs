//! Terminal HTTP tracing and status-painted log formatters.
//!
//! Standard `TraceLayer` logs every response at the same level.
//! This module paints status codes by class in terminal output, while mapping:
//! - 5xx -> ERROR (red)
//! - 429 -> WARN (magenta)
//! - 4xx -> DEBUG (yellow)
//! - 3xx -> DEBUG (cyan)
//! - 2xx -> INFO (green)
//!
//! Color is decided once, honoring `NO_COLOR`, `FORCE_COLOR`, and terminal detection.

use std::sync::OnceLock;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Response, StatusCode};
use tracing::Span;

static COLOUR: OnceLock<bool> = OnceLock::new();

/// Decide whether request logs are painted. Call once at startup.
///
/// Pass `false` for machine-consumed JSON output.
pub fn init_colour(enabled: bool) {
    let _ = COLOUR.set(enabled);
}

fn colour_enabled() -> bool {
    *COLOUR.get_or_init(terminal_supports_colour)
}

/// Whether the stream being written to can show ANSI colour.
///
/// Order of evaluation:
/// 1. `NO_COLOR` environment variable (if present, disable colour).
/// 2. `FORCE_COLOR` or `CLICOLOR_FORCE` (if non-zero and not "false", enable colour).
/// 3. Standard output terminal check (`std::io::stdout().is_terminal()`).
#[must_use]
pub fn terminal_supports_colour() -> bool {
    use std::io::IsTerminal as _;

    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if forced("FORCE_COLOR") || forced("CLICOLOR_FORCE") {
        return true;
    }
    std::io::stdout().is_terminal()
}

fn forced(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| {
        let value = value.trim();
        !value.is_empty() && value != "0" && !value.eq_ignore_ascii_case("false")
    })
}

fn shade(status: StatusCode) -> &'static str {
    let code = status.as_u16();
    if status.is_server_error() {
        "31" // red
    } else if code == 429 {
        "35" // magenta
    } else if status.is_client_error() {
        "33" // yellow
    } else if status.is_redirection() {
        "36" // cyan
    } else {
        "32" // green
    }
}

/// Format the status code, painted with ANSI escapes if colour is enabled.
#[must_use]
pub fn painted(status: StatusCode) -> String {
    let code = status.as_u16();
    if !colour_enabled() {
        return code.to_string();
    }
    format!("\u{1b}[{}m{code}\u{1b}[0m", shade(status))
}

/// Format duration in a compact, human-readable string.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1000 {
        format!("{micros}µs")
    } else if micros < 1_000_000 {
        format!("{:.2}ms", duration.as_secs_f64() * 1000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

/// Log a completed response at a level and colour matching its status code.
///
/// Intended to be wired into `TraceLayer::new_for_http().on_response(on_response)`.
pub fn on_response(response: &Response<Body>, latency: Duration, _span: &Span) {
    let status = response.status();
    let shown = painted(status);
    let ms = latency.as_millis();
    let formatted_latency = format_duration(latency);

    if status.is_server_error() {
        tracing::error!(
            status = %shown,
            latency_ms = ms,
            latency = %formatted_latency,
            "request failed"
        );
    } else if status == StatusCode::TOO_MANY_REQUESTS {
        tracing::warn!(
            status = %shown,
            latency_ms = ms,
            latency = %formatted_latency,
            "request throttled"
        );
    } else if status.is_client_error() || status.is_redirection() {
        tracing::debug!(
            status = %shown,
            latency_ms = ms,
            latency = %formatted_latency,
            "request refused"
        );
    } else {
        tracing::info!(
            status = %shown,
            latency_ms = ms,
            latency = %formatted_latency,
            "request completed"
        );
    }
}
