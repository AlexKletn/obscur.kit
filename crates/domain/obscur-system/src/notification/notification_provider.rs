#![doc = include_str!("notification_provider.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::notification::Notification;

#[doc = include_str!("notification_provider.doc/NotificationResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationResult {
    #[doc = include_str!("notification_provider.doc/Delivered.md")]
    Delivered,
    #[doc = include_str!("notification_provider.doc/Denied.md")]
    Denied,
    #[doc = include_str!("notification_provider.doc/Unsupported.md")]
    Unsupported,
}

impl NotificationResult {
    #[doc = include_str!("notification_provider.doc/is_delivered.md")]
    #[must_use]
    pub const fn is_delivered(&self) -> bool {
        matches!(self, Self::Delivered)
    }
}

#[doc = include_str!("notification_provider.doc/NotificationProvider.md")]
pub trait NotificationProvider {
    #[doc = include_str!("notification_provider.doc/permission.md")]
    #[must_use]
    fn permission(&self) -> SupportLevel;

    #[doc = include_str!("notification_provider.doc/post.md")]
    fn post(&mut self, notification: Notification) -> NotificationResult;

    #[doc = include_str!("notification_provider.doc/set_badge.md")]
    fn set_badge(&mut self, count: Option<u32>);
}

#[doc = include_str!("notification_provider.doc/NullNotificationProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct NullNotificationProvider;

impl NotificationProvider for NullNotificationProvider {
    fn permission(&self) -> SupportLevel {
        // Honest: a null provider offers no notification facility.
        SupportLevel::Unavailable
    }

    fn post(&mut self, _notification: Notification) -> NotificationResult {
        NotificationResult::Unsupported
    }

    fn set_badge(&mut self, _count: Option<u32>) {
        // No-op: there is no app icon to badge.
    }
}

#[doc = include_str!("notification_provider.doc/RecordingNotificationProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct RecordingNotificationProvider {
    posted: Vec<Notification>,
    badge: Option<u32>,
}

impl RecordingNotificationProvider {
    #[doc = include_str!("notification_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("notification_provider.doc/posted.md")]
    #[must_use]
    pub fn posted(&self) -> &[Notification] {
        &self.posted
    }

    #[doc = include_str!("notification_provider.doc/badge.md")]
    #[must_use]
    pub const fn badge(&self) -> Option<u32> {
        self.badge
    }
}

impl NotificationProvider for RecordingNotificationProvider {
    fn permission(&self) -> SupportLevel {
        SupportLevel::Native
    }

    fn post(&mut self, notification: Notification) -> NotificationResult {
        self.posted.push(notification);
        NotificationResult::Delivered
    }

    fn set_badge(&mut self, count: Option<u32>) {
        self.badge = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_provider_is_object_safe() {
        let _: Box<dyn NotificationProvider> = Box::new(NullNotificationProvider);
    }

    #[test]
    fn is_delivered_only_for_delivered() {
        assert!(NotificationResult::Delivered.is_delivered());
        assert!(!NotificationResult::Denied.is_delivered());
        assert!(!NotificationResult::Unsupported.is_delivered());
    }

    #[test]
    fn null_provider_presents_nothing() {
        let mut provider = NullNotificationProvider;
        assert_eq!(provider.permission(), SupportLevel::Unavailable);
        assert_eq!(
            provider.post(Notification::new("t", "b")),
            NotificationResult::Unsupported
        );
        // set_badge is a no-op and must not panic.
        provider.set_badge(Some(5));
        provider.set_badge(None);
    }

    #[test]
    fn recording_provider_records_and_delivers() {
        let mut provider = RecordingNotificationProvider::new();
        assert_eq!(provider.permission(), SupportLevel::Native);
        assert!(provider.posted().is_empty());
        assert_eq!(provider.badge(), None);

        let note = Notification::new("Sync", "Done");
        assert_eq!(provider.post(note.clone()), NotificationResult::Delivered);
        assert_eq!(
            provider.post(Notification::new("Two", "Body")),
            NotificationResult::Delivered
        );
        assert_eq!(provider.posted().len(), 2);
        assert_eq!(provider.posted()[0], note);

        provider.set_badge(Some(7));
        assert_eq!(provider.badge(), Some(7));
        provider.set_badge(None);
        assert_eq!(provider.badge(), None);
    }
}
