use chrono::{Timelike, Utc};
use cms_infra::queue::JobType;
use cms_scheduler::{
    next_run_time, normalize_cron_expr, parse_schedule, CronJobSpec, JobRegistry,
};

#[test]
fn test_normalize_5_part_cron_expr() {
    let normalized = normalize_cron_expr("*/15 * * * *");
    assert_eq!(normalized, "0 */15 * * * *");
}

#[test]
fn test_normalize_6_part_cron_expr() {
    let normalized = normalize_cron_expr("0 30 * * * *");
    assert_eq!(normalized, "0 30 * * * *");
}

#[test]
fn test_cron_parsing_and_next_run() {
    let schedule = parse_schedule("0 0 * * * *").expect("Schedule should parse");
    let now = Utc::now();
    let next = next_run_time(&schedule, now).expect("Next run time should be calculated");

    assert!(next > now);
    assert_eq!(next.minute(), 0);
    assert_eq!(next.second(), 0);
}

#[test]
fn test_invalid_cron_expression_fails_gracefully() {
    let result = parse_schedule("invalid * cron * expression * here");
    assert!(result.is_err());
}

#[test]
fn test_default_registry_jobs() {
    let registry = JobRegistry::default_registry();
    let ids: Vec<&str> = registry.jobs.iter().map(|j| j.id.as_str()).collect();

    assert!(ids.contains(&"session_purge"));
    assert!(ids.contains(&"token_purge"));
    assert!(ids.contains(&"invitation_expire"));
    assert!(ids.contains(&"search_reindex_audit"));
}

#[test]
fn test_custom_job_spec_builder() {
    let spec = CronJobSpec::new("custom_report", "0 0 12 * * *", JobType::Export)
        .with_payload(serde_json::json!({ "format": "pdf", "scope": "full" }));

    assert_eq!(spec.id, "custom_report");
    assert_eq!(spec.cron_expr, "0 0 12 * * *");
    assert_eq!(spec.job_type, JobType::Export);
    assert_eq!(spec.payload["format"], "pdf");
    assert!(spec.enabled);
}
