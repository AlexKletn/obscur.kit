use obscur_capability::SupportLevel;

use super::{Notification, NotificationProvider, NotificationResult};

#[doc = include_str!("../../docs/api/notification/notification_provider.doc/RecordingNotificationProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct RecordingNotificationProvider {
    posted: Vec<Notification>,
    badge: Option<u32>,
}

impl RecordingNotificationProvider {
    /// Creates an empty recording provider (spec §6.12).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the notifications posted so far, in order (spec §6.12).
    #[must_use]
    pub fn posted(&self) -> &[Notification] {
        &self.posted
    }

    /// Returns the current recorded app-icon badge value (spec §6.12).
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
