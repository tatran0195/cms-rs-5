use async_trait::async_trait;
use cms_infra::mailer::{Locale, RenderedEmail};
use serde::{Deserialize, Serialize};

use crate::handler::{Handler, HandlerCtx, HandlerError};

/// Payload schema accepted by the `EmailHandler`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailPayload {
    /// Recipient email address.
    pub to: String,

    /// Optional explicit subject line (overridden if `email` variant is supplied).
    pub subject: Option<String>,

    /// Plain text body (optional if `email` is supplied).
    pub text: Option<String>,

    /// HTML body (optional).
    pub html: Option<String>,

    /// Localized typed email descriptor from `cms_infra::mailer`.
    pub email: Option<cms_infra::mailer::Email>,

    /// Recipient locale tag (e.g. "en", "ja"). Defaults to "en".
    pub locale: Option<String>,
}

/// Outbox handler for dispatching transactional emails via `cms_mailer`.
pub struct EmailHandler {
    event_type: &'static str,
}

impl EmailHandler {
    /// Create an email handler bound to a specific outbox event type.
    pub fn new(event_type: &'static str) -> Self {
        Self { event_type }
    }

    /// Default handler for generic "email.send" events.
    pub fn default_sender() -> Self {
        Self::new("email.send")
    }

    /// Handler for email verification token events.
    pub fn verification() -> Self {
        Self::new("email.verification")
    }

    /// Handler for workspace invitation events.
    pub fn invitation() -> Self {
        Self::new("email.invitation")
    }
}

#[async_trait]
impl Handler for EmailHandler {
    fn event_type(&self) -> &'static str {
        self.event_type
    }

    async fn handle(
        &self,
        ctx: &HandlerCtx,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, HandlerError> {
        let mailer = ctx.mailer.as_ref().ok_or_else(|| {
            HandlerError::Transient("No Mailer configured in HandlerCtx".to_string())
        })?;

        let email_payload: EmailPayload = serde_json::from_value(payload.clone()).map_err(|e| {
            HandlerError::Permanent(format!("Malformed EmailPayload: {e}"))
        })?;

        if email_payload.to.trim().is_empty() {
            return Err(HandlerError::Permanent("Recipient address is empty".to_string()));
        }

        let rendered: RenderedEmail = if let Some(ref email_variant) = email_payload.email {
            let renderer = ctx.renderer.as_ref().ok_or_else(|| {
                HandlerError::Permanent("EmailRenderer is required to render typed Email variant".to_string())
            })?;

            let locale = match email_payload.locale.as_deref() {
                Some(loc_str) => Locale::parse(loc_str).unwrap_or_else(|_| Locale::default()),
                None => Locale::default(),
            };

            renderer.render(email_variant, &locale).map_err(|e| {
                HandlerError::Permanent(format!("Failed to render email template: {e}"))
            })?
        } else {
            let subject = email_payload.subject.unwrap_or_default();
            let text = email_payload.text.unwrap_or_default();
            let html = email_payload.html.unwrap_or_else(|| {
                let escaped = text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('\n', "<br/>");
                format!("<p>{escaped}</p>")
            });

            if subject.is_empty() && text.is_empty() {
                return Err(HandlerError::Permanent(
                    "Email must provide either a typed 'email' variant or 'subject' and 'text'".to_string()
                ));
            }

            RenderedEmail { subject, html, text }
        };

        mailer.send_rendered_email(&email_payload.to, &rendered).await.map_err(|e| {
            // Categorize transport errors: connection/timeout errors are transient,
            // recipient or authentication rejections may be permanent.
            let err_str = e.to_string();
            if err_str.contains("timeout") || err_str.contains("connection") || err_str.contains("temporarily") {
                HandlerError::Transient(err_str)
            } else {
                HandlerError::Permanent(err_str)
            }
        })?;

        Ok(serde_json::json!({
            "status": "sent",
            "to": email_payload.to,
            "subject": rendered.subject
        }))
    }
}
