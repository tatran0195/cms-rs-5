use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use sqlx::PgPool;
use uuid::Uuid;

/// Execution context passed to an outbox handler.
#[derive(Clone)]
pub struct HandlerCtx {
    pub event_id: Uuid,
    pub workspace_id: Option<String>,
    pub attempts: i32,
    pub pool: PgPool,
    pub http_client: reqwest::Client,
    pub mailer: Option<Arc<dyn cms_infra::mailer::Mailer>>,
    pub renderer: Option<cms_infra::mailer::EmailRenderer>,
}

impl HandlerCtx {
    /// Create a new handler context with standard infrastructure components.
    pub fn new(
        event_id: Uuid,
        workspace_id: Option<String>,
        attempts: i32,
        pool: PgPool,
        http_client: reqwest::Client,
        mailer: Option<Arc<dyn cms_infra::mailer::Mailer>>,
        renderer: Option<cms_infra::mailer::EmailRenderer>,
    ) -> Self {
        Self {
            event_id,
            workspace_id,
            attempts,
            pool,
            http_client,
            mailer,
            renderer,
        }
    }
}

/// Categorized outbox handler error distinguishing retryable transient issues
/// from terminal permanent failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum HandlerError {
    #[error("Transient failure: {0}")]
    Transient(String),

    #[error("Permanent failure: {0}")]
    Permanent(String),
}

/// Trait implemented by outbox event processors.
#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    /// The event type string this handler processes (e.g. "email.verification", "webhook.post").
    fn event_type(&self) -> &'static str;

    /// Process the outbox event payload.
    async fn handle(
        &self,
        ctx: &HandlerCtx,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, HandlerError>;
}

/// Thread-safe registry holding event handlers.
#[derive(Default, Clone)]
pub struct Registry {
    handlers: Arc<RwLock<HashMap<&'static str, Arc<dyn Handler>>>>,
}

impl Registry {
    /// Create an empty handler registry.
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a typed event handler.
    pub fn register<H: Handler + 'static>(&self, handler: H) {
        let key = handler.event_type();
        self.handlers.write().insert(key, Arc::new(handler));
    }

    /// Register an Arc-wrapped event handler.
    pub fn register_arc(&self, handler: Arc<dyn Handler>) {
        let key = handler.event_type();
        self.handlers.write().insert(key, handler);
    }

    /// Check if a handler is registered for a given event type.
    pub fn has(&self, event_type: &str) -> bool {
        self.handlers.read().contains_key(event_type)
    }

    /// Retrieve a registered handler for the given event type.
    pub fn get(&self, event_type: &str) -> Option<Arc<dyn Handler>> {
        self.handlers.read().get(event_type).cloned()
    }

    /// List all registered event types.
    pub fn registered_types(&self) -> Vec<&'static str> {
        self.handlers.read().keys().copied().collect()
    }

    /// Dispatch an event to its registered handler.
    pub async fn dispatch(
        &self,
        ctx: &HandlerCtx,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, HandlerError> {
        let handler = self.get(event_type).ok_or_else(|| {
            HandlerError::Permanent(format!("No handler registered for event type: '{event_type}'"))
        })?;

        handler.handle(ctx, payload).await
    }
}
