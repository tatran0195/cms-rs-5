use chrono::Utc;
use cms_entity::notification::{
    Notification, NotificationResponse, NotificationStatus, NotificationType,
};
use cms_notifications::{
    hub::{NotificationEvent, NotificationHub},
    queries::NotificationRow,
};
use serde_json::json;

#[tokio::test]
async fn test_notification_hub_subscribe_and_publish() {
    let hub = NotificationHub::new();
    let user_id = "user-123";

    let mut rx1 = hub.subscribe(user_id);
    let mut rx2 = hub.subscribe(user_id);

    assert_eq!(hub.active_subscribers(user_id), 2);
    assert_eq!(hub.active_users(), 1);

    let event = NotificationEvent::UnreadCount { unread_count: 5 };
    let delivered = hub.publish(user_id, event);
    assert_eq!(delivered, 2);

    let received1 = rx1.recv().await.expect("rx1 should receive event");
    let received2 = rx2.recv().await.expect("rx2 should receive event");

    match received1 {
        NotificationEvent::UnreadCount { unread_count } => assert_eq!(unread_count, 5),
        _ => panic!("Expected UnreadCount event"),
    }
    match received2 {
        NotificationEvent::UnreadCount { unread_count } => assert_eq!(unread_count, 5),
        _ => panic!("Expected UnreadCount event"),
    }

    drop(rx1);
    drop(rx2);

    hub.clean_idle_channels();
    assert_eq!(hub.active_subscribers(user_id), 0);
    assert_eq!(hub.active_users(), 0);
}

#[tokio::test]
async fn test_notification_hub_user_isolation() {
    let hub = NotificationHub::new();
    let mut rx_alice = hub.subscribe("alice");
    let mut rx_bob = hub.subscribe("bob");

    let event_alice = NotificationEvent::UnreadCount { unread_count: 10 };
    hub.publish("alice", event_alice);

    let received_alice = rx_alice.recv().await.expect("alice should receive");
    match received_alice {
        NotificationEvent::UnreadCount { unread_count } => assert_eq!(unread_count, 10),
        _ => panic!("Expected UnreadCount event"),
    }

    // Bob should not receive alice's event
    assert!(rx_bob.try_recv().is_err());
}

#[test]
fn test_notification_event_serde() {
    let now = Utc::now();
    let sample = NotificationResponse {
        id: "notif-1".to_string(),
        user_id: "user-1".to_string(),
        notification_type: NotificationType::Mention,
        title: "Mentioned in doc".to_string(),
        message: "You were mentioned".to_string(),
        data: json!({ "doc_id": "doc-abc" }),
        status: NotificationStatus::Unread,
        read_at: None,
        created_at: now,
        updated_at: now,
    };

    let event = NotificationEvent::Created {
        notification: sample,
        unread_count: 1,
    };

    let serialized = serde_json::to_string(&event).expect("must serialize");
    assert!(serialized.contains("\"type\":\"created\""));
    assert!(serialized.contains("Mentioned in doc"));

    let deserialized: NotificationEvent =
        serde_json::from_str(&serialized).expect("must deserialize");
    match deserialized {
        NotificationEvent::Created {
            notification,
            unread_count,
        } => {
            assert_eq!(notification.id, "notif-1");
            assert_eq!(unread_count, 1);
        }
        _ => panic!("Wrong variant deserialized"),
    }
}

#[test]
fn test_notification_row_conversions() {
    let now = Utc::now();
    let row = NotificationRow {
        id: "id-1".to_string(),
        user_id: "u-1".to_string(),
        notification_type: NotificationType::Comment,
        title: "New Comment".to_string(),
        message: "Alice commented".to_string(),
        data: json!({}),
        status: NotificationStatus::Unread,
        read_at: None,
        created_at: now,
        updated_at: now,
    };

    let notif: Notification = row.into();
    assert_eq!(notif.id, "id-1");
    assert_eq!(notif.title, "New Comment");

    let row2 = NotificationRow {
        id: "id-2".to_string(),
        user_id: "u-2".to_string(),
        notification_type: NotificationType::System,
        title: "System Update".to_string(),
        message: "Maintenance scheduled".to_string(),
        data: json!({}),
        status: NotificationStatus::Read,
        read_at: Some(now),
        created_at: now,
        updated_at: now,
    };

    let resp: NotificationResponse = row2.into();
    assert_eq!(resp.id, "id-2");
    assert_eq!(resp.status, NotificationStatus::Read);
}
