use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};

use crate::handler::{Handler, HandlerCtx, HandlerError};

/// Schema for outgoing webhook event payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Destination endpoint URL.
    pub url: String,

    /// HTTP request method (e.g. "POST", "PUT"). Defaults to "POST".
    #[serde(default = "default_method")]
    pub method: String,

    /// Custom HTTP headers to include with the request.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// JSON request body.
    pub body: Option<serde_json::Value>,

    /// Optional per-request timeout in seconds (default: 30s).
    pub timeout_seconds: Option<u64>,
}

fn default_method() -> String {
    "POST".to_string()
}

/// Outbox handler for dispatching outgoing webhooks over HTTP.
pub struct WebhookHandler {
    event_type: &'static str,
}

impl WebhookHandler {
    /// Create a webhook handler bound to a specific outbox event type.
    pub fn new(event_type: &'static str) -> Self {
        Self { event_type }
    }

    /// Default handler for generic "webhook.post" events.
    pub fn default_poster() -> Self {
        Self::new("webhook.post")
    }
}

#[async_trait]
impl Handler for WebhookHandler {
    fn event_type(&self) -> &'static str {
        self.event_type
    }

    async fn handle(
        &self,
        ctx: &HandlerCtx,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, HandlerError> {
        let webhook_payload: WebhookPayload =
            serde_json::from_value(payload.clone()).map_err(|e| {
                HandlerError::Permanent(format!("Malformed WebhookPayload: {e}"))
            })?;

        let parsed_url = Url::parse(&webhook_payload.url).map_err(|e| {
            HandlerError::Permanent(format!("Invalid webhook URL '{}': {e}", webhook_payload.url))
        })?;

        let method = webhook_payload.method.parse::<Method>().map_err(|e| {
            HandlerError::Permanent(format!("Invalid HTTP method '{}': {e}", webhook_payload.method))
        })?;

        let timeout = Duration::from_secs(webhook_payload.timeout_seconds.unwrap_or(30));

        let mut request_builder = ctx
            .http_client
            .request(method, parsed_url)
            .timeout(timeout);

        let mut has_content_type = false;
        for (key, val) in &webhook_payload.headers {
            if key.eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            request_builder = request_builder.header(key, val);
        }

        if let Some(ref body) = webhook_payload.body {
            if !has_content_type {
                request_builder = request_builder.header("Content-Type", "application/json");
            }
            request_builder = request_builder.json(body);
        }

        let response = request_builder.send().await.map_err(|e| {
            if e.is_timeout() || e.is_connect() {
                HandlerError::Transient(format!("Webhook connection/timeout failure: {e}"))
            } else {
                HandlerError::Permanent(format!("Webhook request construction failed: {e}"))
            }
        })?;

        let status = response.status();
        if status.is_success() {
            let body_preview = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            let truncated = if body_preview.len() > 500 {
                format!("{}...", &body_preview[..500])
            } else {
                body_preview
            };

            Ok(serde_json::json!({
                "status": status.as_u16(),
                "response": truncated
            }))
        } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            Err(HandlerError::Transient(format!(
                "Webhook endpoint returned retryable status: {status}"
            )))
        } else {
            Err(HandlerError::Permanent(format!(
                "Webhook endpoint rejected request with status: {status}"
            )))
        }
    }
}
