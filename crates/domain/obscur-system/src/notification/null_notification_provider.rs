use obscur_capability::SupportLevel;

use super::{Notification, NotificationProvider, NotificationResult};

#[doc = include_str!("../../docs/api/notification/notification_provider.doc/NullNotificationProvider.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
