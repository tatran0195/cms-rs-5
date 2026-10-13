//! Real-time notification hub for WebSockets and live feeds

use std::{collections::HashMap, sync::Arc};

use cms_entity::notification::NotificationResponse;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events broadcast to connected clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationEvent {
    /// New notification received
    Created {
        notification: NotificationResponse,
        unread_count: i64,
    },
    /// Notification(s) marked as read
    Read {
        notification_ids: Vec<String>,
        unread_count: i64,
    },
    /// All notifications marked as read
    AllRead { unread_count: i64 },
    /// Notification archived
    Archived {
        notification_id: String,
        unread_count: i64,
    },
    /// Unread count update
    UnreadCount { unread_count: i64 },
    /// Heartbeat ping from server
    Ping,
}

/// Notification hub for managing per-user WebSocket channels
#[derive(Clone, Default)]
pub struct NotificationHub {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<NotificationEvent>>>>,
}

impl NotificationHub {
    pub const CHANNEL_CAPACITY: usize = 64;

    /// Create a new notification hub
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to real-time events for a specific user.
    pub fn subscribe(&self, user_id: &str) -> broadcast::Receiver<NotificationEvent> {
        let mut channels = self.channels.write();
        if let Some(sender) = channels.get(user_id) {
            sender.subscribe()
        } else {
            let (sender, receiver) = broadcast::channel(Self::CHANNEL_CAPACITY);
            channels.insert(user_id.to_string(), sender);
            receiver
        }
    }

    /// Publish an event to all active subscribers of a specific user.
    /// Returns the number of active receivers that received the event.
    pub fn publish(&self, user_id: &str, event: NotificationEvent) -> usize {
        let channels = self.channels.read();
        channels
            .get(user_id)
            .map_or(0, |sender| sender.send(event).unwrap_or_default())
    }

    /// Return the number of active subscribers for a specific user.
    pub fn active_subscribers(&self, user_id: &str) -> usize {
        let channels = self.channels.read();
        channels.get(user_id).map_or(0, |s| s.receiver_count())
    }

    /// Return total active unique users currently subscribed.
    pub fn active_users(&self) -> usize {
        let channels = self.channels.read();
        channels.values().filter(|s| s.receiver_count() > 0).count()
    }

    /// Clean up channels that have zero active receivers.
    pub fn clean_idle_channels(&self) {
        let mut channels = self.channels.write();
        channels.retain(|_, sender| sender.receiver_count() > 0);
    }
}
