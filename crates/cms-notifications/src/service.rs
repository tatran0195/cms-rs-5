//! Notification service layer coordinating DB and real-time hub

use std::sync::Arc;
use cms_entity::{
    common::PaginatedResponse,
    notification::{
        NotificationCountResponse, NotificationResponse, NotificationStatus, NotificationType,
    },
};
use cms_error::AppError;
use sqlx::PgPool;

use crate::{
    hub::{NotificationEvent, NotificationHub},
    queries::NotificationQueries,
};

/// Notification service coordinating business logic, persistence, and WebSocket fan-out
pub struct NotificationService;

impl NotificationService {
    /// Create a notification, persist to DB, and publish to WebSocket subscribers
    pub async fn create_notification(
        pool: &PgPool,
        hub: &Arc<NotificationHub>,
        user_id: &str,
        notification_type: NotificationType,
        title: &str,
        message: &str,
        data: serde_json::Value,
    ) -> Result<NotificationResponse, AppError> {
        let notification = NotificationQueries::create(
            pool,
            user_id,
            notification_type,
            title,
            message,
            data,
        )
        .await?;

        let response: NotificationResponse = notification.into();
        let unread_count = NotificationQueries::count_unread_by_user(pool, user_id).await?;

        hub.publish(
            user_id,
            NotificationEvent::Created {
                notification: response.clone(),
                unread_count,
            },
        );

        Ok(response)
    }

    /// Retrieve a notification by ID with caller ownership verification
    pub async fn get_notification(
        pool: &PgPool,
        user_id: &str,
        notification_id: &str,
    ) -> Result<NotificationResponse, AppError> {
        let notification = NotificationQueries::get_by_id(pool, notification_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

        if notification.user_id != user_id {
            return Err(AppError::AccessDenied(
                "Notification does not belong to this user".to_string(),
            ));
        }

        Ok(notification.into())
    }

    /// List paginated notifications for a user
    pub async fn list_notifications(
        pool: &PgPool,
        user_id: &str,
        status: Option<NotificationStatus>,
        page: u64,
        page_size: u64,
    ) -> Result<PaginatedResponse<NotificationResponse>, AppError> {
        let limit = page_size.max(1) as i64;
        let offset = page.saturating_sub(1) as i64 * limit;

        let notifications = NotificationQueries::get_by_user(
            pool,
            user_id,
            status.as_ref(),
            Some(limit),
            Some(offset),
        )
        .await?;

        let total = NotificationQueries::count_by_user(pool, user_id, status.as_ref()).await?;

        Ok(PaginatedResponse::new(
            notifications.into_iter().map(Into::into).collect(),
            total as u64,
            page,
            page_size,
        ))
    }

    /// Mark a single notification as read and publish event to hub
    pub async fn mark_as_read(
        pool: &PgPool,
        hub: &Arc<NotificationHub>,
        user_id: &str,
        notification_id: &str,
    ) -> Result<NotificationResponse, AppError> {
        let notification = NotificationQueries::get_by_id(pool, notification_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

        if notification.user_id != user_id {
            return Err(AppError::AccessDenied(
                "Notification does not belong to this user".to_string(),
            ));
        }

        let updated =
            NotificationQueries::update_status(pool, notification_id, NotificationStatus::Read)
                .await?;

        let unread_count = NotificationQueries::count_unread_by_user(pool, user_id).await?;
        hub.publish(
            user_id,
            NotificationEvent::Read {
                notification_ids: vec![notification_id.to_string()],
                unread_count,
            },
        );

        Ok(updated.into())
    }

    /// Mark multiple notifications as read and publish event to hub
    pub async fn mark_notifications_read(
        pool: &PgPool,
        hub: &Arc<NotificationHub>,
        user_id: &str,
        notification_ids: &[String],
    ) -> Result<(), AppError> {
        let mut successfully_marked = Vec::new();
        for id in notification_ids {
            if let Ok(Some(n)) = NotificationQueries::get_by_id(pool, id).await {
                if n.user_id == user_id
                    && NotificationQueries::update_status(pool, id, NotificationStatus::Read)
                        .await
                        .is_ok()
                {
                    successfully_marked.push(id.clone());
                }
            }
        }

        let unread_count = NotificationQueries::count_unread_by_user(pool, user_id).await?;
        if !successfully_marked.is_empty() {
            hub.publish(
                user_id,
                NotificationEvent::Read {
                    notification_ids: successfully_marked,
                    unread_count,
                },
            );
        }

        Ok(())
    }

    /// Mark all notifications as read for a user and publish event to hub
    pub async fn mark_all_as_read(
        pool: &PgPool,
        hub: &Arc<NotificationHub>,
        user_id: &str,
    ) -> Result<u64, AppError> {
        let count = NotificationQueries::mark_all_as_read(pool, user_id).await?;
        hub.publish(user_id, NotificationEvent::AllRead { unread_count: 0 });
        Ok(count)
    }

    /// Archive a notification and publish event to hub
    pub async fn archive_notification(
        pool: &PgPool,
        hub: &Arc<NotificationHub>,
        user_id: &str,
        notification_id: &str,
    ) -> Result<(), AppError> {
        let notification = NotificationQueries::get_by_id(pool, notification_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

        if notification.user_id != user_id {
            return Err(AppError::AccessDenied(
                "Notification does not belong to this user".to_string(),
            ));
        }

        NotificationQueries::update_status(pool, notification_id, NotificationStatus::Archived)
            .await?;

        let unread_count = NotificationQueries::count_unread_by_user(pool, user_id).await?;
        hub.publish(
            user_id,
            NotificationEvent::Archived {
                notification_id: notification_id.to_string(),
                unread_count,
            },
        );

        Ok(())
    }

    /// Delete a notification
    pub async fn delete_notification(
        pool: &PgPool,
        hub: &Arc<NotificationHub>,
        user_id: &str,
        notification_id: &str,
    ) -> Result<bool, AppError> {
        let notification = NotificationQueries::get_by_id(pool, notification_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

        if notification.user_id != user_id {
            return Err(AppError::AccessDenied(
                "Notification does not belong to this user".to_string(),
            ));
        }

        let deleted = NotificationQueries::delete(pool, notification_id).await?;
        if deleted {
            let unread_count = NotificationQueries::count_unread_by_user(pool, user_id).await?;
            hub.publish(user_id, NotificationEvent::UnreadCount { unread_count });
        }

        Ok(deleted)
    }

    /// Get unread notification count
    pub async fn get_unread_count(pool: &PgPool, user_id: &str) -> Result<i64, AppError> {
        NotificationQueries::count_unread_by_user(pool, user_id).await
    }

    /// Get notification count breakdown (total, unread, by_type)
    pub async fn get_notification_count(
        pool: &PgPool,
        user_id: &str,
    ) -> Result<NotificationCountResponse, AppError> {
        let total = NotificationQueries::count_by_user(pool, user_id, None).await?;
        let unread = NotificationQueries::count_unread_by_user(pool, user_id).await?;
        let by_type = NotificationQueries::count_by_type(pool, user_id).await?;

        Ok(NotificationCountResponse {
            total,
            unread,
            by_type,
        })
    }
}
