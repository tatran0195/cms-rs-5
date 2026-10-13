use std::str::FromStr;

use chrono::{DateTime, Utc};
use cms_error::AppError;
use cron::Schedule;

/// Normalize cron expression to 6 or 7 fields (prepending "0 " for seconds if only 5 fields provided).
pub fn normalize_cron_expr(expr: &str) -> String {
    if expr.split_whitespace().count() == 5 {
        format!("0 {expr}")
    } else {
        expr.to_string()
    }
}

/// Parse a cron expression into a compiled Schedule instance.
pub fn parse_schedule(expr: &str) -> Result<Schedule, AppError> {
    let normalized = normalize_cron_expr(expr.trim());
    Schedule::from_str(&normalized)
        .map_err(|e| AppError::InvalidInput(format!("Invalid cron expression '{expr}': {e}")))
}

/// Calculate the next scheduled execution timestamp after a given point in time.
pub fn next_run_time(schedule: &Schedule, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    schedule.after(&after).next()
}
