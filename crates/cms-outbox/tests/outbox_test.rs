use async_trait::async_trait;
use chrono::Utc;
use cms_outbox::{
    calculate_backoff,
    handler::{Handler, HandlerCtx, HandlerError, Registry},
    handlers::{email::EmailPayload, webhook::WebhookPayload},
    BackoffConfig, OutboxRow,
};
use serde_json::json;
use uuid::Uuid;

struct DummyHandler {
    event_type: &'static str,
    should_fail_permanent: bool,
    should_fail_transient: bool,
}

#[async_trait]
impl Handler for DummyHandler {
    fn event_type(&self) -> &'static str {
        self.event_type
    }

    async fn handle(
        &self,
        _ctx: &HandlerCtx,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, HandlerError> {
        if self.should_fail_permanent {
            return Err(HandlerError::Permanent("test permanent failure".to_string()));
        }
        if self.should_fail_transient {
            return Err(HandlerError::Transient("test transient failure".to_string()));
        }
        Ok(json!({ "processed": true, "input": payload }))
    }
}

#[test]
fn test_backoff_growth_and_caps() {
    let config = BackoffConfig::default();

    let d0 = config.calculate_deterministic(0, 0.0);
    assert_eq!(d0.num_seconds(), 1);

    let d1 = config.calculate_deterministic(1, 0.0);
    assert_eq!(d1.num_seconds(), 2);

    let d2 = config.calculate_deterministic(2, 0.0);
    assert_eq!(d2.num_seconds(), 4);

    let d3 = config.calculate_deterministic(3, 0.0);
    assert_eq!(d3.num_seconds(), 8);

    let d4 = config.calculate_deterministic(4, 0.0);
    assert_eq!(d4.num_seconds(), 16);

    // High attempt capped at max_seconds (3600)
    let d20 = config.calculate_deterministic(20, 0.0);
    assert_eq!(d20.num_seconds(), 3600);

    // Negative attempt defaults gracefully to initial_seconds
    let d_neg = config.calculate_deterministic(-10, 0.0);
    assert_eq!(d_neg.num_seconds(), 1);

    // Convenience function runs without error
    let d_rand = calculate_backoff(3);
    assert!(d_rand.num_seconds() >= 1);
}

#[test]
fn test_outbox_row_builder() {
    let test_id = Uuid::new_v4();
    let scheduled = Utc::now();
    let row = OutboxRow::new("test.event", json!({ "key": "value" }))
        .with_id(test_id)
        .with_workspace("ws_123")
        .with_max_attempts(5)
        .with_scheduled_at(scheduled);

    assert_eq!(row.id, Some(test_id));
    assert_eq!(row.workspace_id.as_deref(), Some("ws_123"));
    assert_eq!(row.event_type, "test.event");
    assert_eq!(row.max_attempts, Some(5));
    assert_eq!(row.scheduled_at, Some(scheduled));
    assert_eq!(row.payload["key"], "value");
}

#[tokio::test]
async fn test_registry_dispatch_flow() {
    let registry = Registry::new();
    assert!(!registry.has("test.success"));

    registry.register(DummyHandler {
        event_type: "test.success",
        should_fail_permanent: false,
        should_fail_transient: false,
    });
    registry.register(DummyHandler {
        event_type: "test.fail.perm",
        should_fail_permanent: true,
        should_fail_transient: false,
    });
    registry.register(DummyHandler {
        event_type: "test.fail.trans",
        should_fail_permanent: false,
        should_fail_transient: true,
    });

    assert!(registry.has("test.success"));
    assert!(registry.has("test.fail.perm"));
    assert!(registry.has("test.fail.trans"));
    assert_eq!(registry.registered_types().len(), 3);

    // Dummy context (no live DB required for purely testing registry handler dispatch)
    let ctx = HandlerCtx {
        event_id: Uuid::new_v4(),
        workspace_id: Some("ws_test".to_string()),
        attempts: 1,
        pool: sqlx::PgPool::connect_lazy("postgres://localhost/dummy").unwrap_or_else(|_| {
            panic!("connect_lazy should construct pool without connecting")
        }),
        http_client: reqwest::Client::new(),
        mailer: None,
        renderer: None,
    };

    // 1. Success dispatch
    let ok_res = registry
        .dispatch(&ctx, "test.success", &json!({ "foo": "bar" }))
        .await
        .expect("handler should succeed");
    assert_eq!(ok_res["processed"], true);
    assert_eq!(ok_res["input"]["foo"], "bar");

    // 2. Permanent error dispatch
    let perm_err = registry
        .dispatch(&ctx, "test.fail.perm", &json!({}))
        .await
        .expect_err("should return permanent error");
    assert_eq!(
        perm_err,
        HandlerError::Permanent("test permanent failure".to_string())
    );

    // 3. Transient error dispatch
    let trans_err = registry
        .dispatch(&ctx, "test.fail.trans", &json!({}))
        .await
        .expect_err("should return transient error");
    assert_eq!(
        trans_err,
        HandlerError::Transient("test transient failure".to_string())
    );

    // 4. Missing handler
    let missing_err = registry
        .dispatch(&ctx, "unregistered.event", &json!({}))
        .await
        .expect_err("should fail on unregistered");
    match missing_err {
        HandlerError::Permanent(msg) => {
            assert!(msg.contains("No handler registered"));
        }
        HandlerError::Transient(_) => panic!("expected permanent failure for missing handler"),
    }
}

#[test]
fn test_email_payload_serde() {
    let payload = EmailPayload {
        to: "developer@example.com".to_string(),
        subject: Some("Test Subject".to_string()),
        text: Some("Test body content".to_string()),
        html: None,
        email: Some(cms_infra::mailer::Email::VerifyEmail {
            url: "https://example.com/verify".to_string(),
        }),
        locale: Some("en".to_string()),
    };

    let serialized = serde_json::to_value(&payload).expect("serialization succeeds");
    assert_eq!(serialized["to"], "developer@example.com");
    assert_eq!(serialized["subject"], "Test Subject");
    assert_eq!(serialized["email"]["kind"], "verify_email");

    let deserialized: EmailPayload =
        serde_json::from_value(serialized).expect("deserialization succeeds");
    assert_eq!(deserialized.to, "developer@example.com");
    assert_eq!(deserialized.locale.as_deref(), Some("en"));
}

#[test]
fn test_webhook_payload_serde() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Api-Key".to_string(), "secret123".to_string());

    let payload = WebhookPayload {
        url: "https://api.internal/webhook".to_string(),
        method: "POST".to_string(),
        headers,
        body: Some(json!({ "event": "page_published", "id": "p1" })),
        timeout_seconds: Some(15),
    };

    let val = serde_json::to_value(&payload).expect("serialization succeeds");
    assert_eq!(val["url"], "https://api.internal/webhook");
    assert_eq!(val["method"], "POST");
    assert_eq!(val["timeout_seconds"], 15);
    assert_eq!(val["headers"]["X-Api-Key"], "secret123");

    let parsed: WebhookPayload = serde_json::from_value(val).expect("deserialization succeeds");
    assert_eq!(parsed.url, "https://api.internal/webhook");
    assert_eq!(parsed.body.unwrap()["event"], "page_published");
}
