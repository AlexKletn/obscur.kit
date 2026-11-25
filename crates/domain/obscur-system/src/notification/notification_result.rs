#[doc = include_str!("../../docs/api/notification/notification_provider.doc/NotificationResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationResult {
    /// The host accepted the notification for presentation.
    Delivered,
    /// The host is capable of notifications but permission policy refused this
    /// one.
    Denied,
    /// The host offers no notification facility.
    Unsupported,
}

impl NotificationResult {
    /// Returns whether the notification was delivered (spec §6.12).
    #[must_use]
    pub const fn is_delivered(&self) -> bool {
        matches!(self, Self::Delivered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_delivered_only_for_delivered() {
        assert!(NotificationResult::Delivered.is_delivered());
        assert!(!NotificationResult::Denied.is_delivered());
        assert!(!NotificationResult::Unsupported.is_delivered());
    }
}
