//! Email business logic and delivery service.

use std::sync::Arc;

use cms_entity::email::{EmailRequest, EmailTemplate};
use cms_error::AppError;

use super::{email::Email, render::EmailRenderer, transport::Mailer, types::RenderedEmail};
use crate::i18n::Locale;

/// Email service
#[derive(Debug, Clone, Copy)]
pub struct EmailService;

impl EmailService {
    /// Send an email (supports both plain text and rich HTML if html_body is present)
    pub async fn send_email(
        mailer: Arc<dyn Mailer>,
        request: EmailRequest,
    ) -> Result<(), AppError> {
        if let Some(html) = request.html_body {
            let rendered = RenderedEmail {
                subject: request.subject,
                text: request.body,
                html,
            };
            mailer.send_rendered_email(&request.to, &rendered).await
        } else {
            mailer
                .send_email(&request.to, &request.subject, &request.body)
                .await
        }
    }

    /// Send a typed, localized transactional email
    pub async fn send_typed_email(
        mailer: Arc<dyn Mailer>,
        renderer: &EmailRenderer,
        to: &str,
        email: &Email,
        locale: Option<&Locale>,
    ) -> Result<(), AppError> {
        let default_loc =
            Locale::parse("en").map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
        let locale = locale.unwrap_or(&default_loc);
        let rendered = renderer
            .render(email, locale)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
        mailer.send_rendered_email(to, &rendered).await
    }

    /// Send a templated email
    pub async fn send_templated_email(
        mailer: Arc<dyn Mailer>,
        template: EmailTemplate,
        to: &str,
        variables: serde_json::Value,
    ) -> Result<(), AppError> {
        let (subject, body, html_body) = Self::render_template(&template, &variables)?;
        if let Some(html) = html_body {
            let rendered = RenderedEmail {
                subject,
                text: body,
                html,
            };
            mailer.send_rendered_email(to, &rendered).await
        } else {
            mailer.send_email(to, &subject, &body).await
        }
    }

    /// Render an email template
    fn render_template(
        template: &EmailTemplate,
        variables: &serde_json::Value,
    ) -> Result<(String, String, Option<String>), AppError> {
        let mut subject = template.subject.clone();
        let mut body = template.body.clone();
        let mut html_body = template.html_body.clone();

        if let serde_json::Value::Object(map) = variables {
            for (key, value) in map {
                let placeholder = format!("{{{}}}", key);
                let replacement = match &value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                };

                subject = subject.replace(&placeholder, &replacement);
                body = body.replace(&placeholder, &replacement);
                if let Some(ref mut html) = html_body {
                    *html = html.replace(&placeholder, &replacement);
                }
            }
        }

        Ok((subject, body, html_body))
    }
}

/// Process email job using an explicit renderer (for worker queue)
pub async fn process_email_job_with_renderer(
    mailer: Arc<dyn Mailer>,
    renderer: &EmailRenderer,
    payload: &serde_json::Value,
) -> Result<(), AppError> {
    let to = payload
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing to address".to_string()))?;

    // Check if payload contains a structured Email object
    if let Some(email_val) = payload.get("email") {
        let email: Email = serde_json::from_value(email_val.clone())
            .map_err(|e| AppError::InvalidInput(format!("Invalid email payload: {e}")))?;
        let locale_str = payload.get("locale").and_then(|v| v.as_str());
        let target_locale = renderer.negotiate(locale_str);
        let rendered = renderer
            .render(&email, &target_locale)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
        return mailer.send_rendered_email(to, &rendered).await;
    }

    let subject = payload
        .get("subject")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing subject".to_string()))?;
    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing body".to_string()))?;

    let html = payload.get("html").and_then(|v| v.as_str());

    if let Some(html) = html {
        let rendered = RenderedEmail {
            subject: subject.to_string(),
            text: body.to_string(),
            html: html.to_string(),
        };
        mailer.send_rendered_email(to, &rendered).await
    } else {
        mailer.send_email(to, subject, body).await
    }
}

/// Process email job with default embedded renderer (for worker queue)
pub async fn process_email_job(
    mailer: Arc<dyn Mailer>,
    payload: &serde_json::Value,
) -> Result<(), AppError> {
    let renderer = EmailRenderer::embedded().map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to initialize email renderer: {e}"))
    })?;
    process_email_job_with_renderer(mailer, &renderer, payload).await
}
