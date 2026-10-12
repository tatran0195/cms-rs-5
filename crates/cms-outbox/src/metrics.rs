/// Record that an outbox event was enqueued.
pub fn record_queued(event_type: &str) {
    metrics::counter!("outbox_events_queued_total", "event_type" => event_type.to_string())
        .increment(1);
}

/// Record that an outbox event finished processing successfully.
pub fn record_completed(event_type: &str, duration_secs: f64) {
    metrics::counter!(
        "outbox_events_processed_total",
        "event_type" => event_type.to_string(),
        "status" => "completed"
    )
    .increment(1);

    metrics::histogram!(
        "outbox_process_duration_seconds",
        "event_type" => event_type.to_string()
    )
    .record(duration_secs);
}

/// Record that an outbox event encountered a transient retryable failure.
pub fn record_retry(event_type: &str, attempts: i32) {
    metrics::counter!(
        "outbox_events_processed_total",
        "event_type" => event_type.to_string(),
        "status" => "retrying"
    )
    .increment(1);

    metrics::histogram!(
        "outbox_event_attempts",
        "event_type" => event_type.to_string()
    )
    .record(attempts as f64);
}

/// Record that an outbox event failed terminally and was moved to the Dead Letter Queue.
pub fn record_dlq(event_type: &str) {
    metrics::counter!(
        "outbox_events_processed_total",
        "event_type" => event_type.to_string(),
        "status" => "dlq"
    )
    .increment(1);

    metrics::counter!("outbox_dlq_total", "event_type" => event_type.to_string()).increment(1);
}
