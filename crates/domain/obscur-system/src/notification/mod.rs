//! Notification module.
mod notification;
pub use notification::Notification;
mod notification_action;
pub use notification_action::NotificationAction;
mod notification_provider;
pub use notification_provider::{
    NotificationProvider, NotificationResult, NullNotificationProvider,
    RecordingNotificationProvider,
};
