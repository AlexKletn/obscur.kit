#![doc = include_str!("../../docs/api/notification/notification_provider.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::notification::Notification;

use super::NotificationResult;

#[doc = include_str!("../../docs/api/notification/notification_provider.doc/NotificationProvider.md")]
pub trait NotificationProvider {
    #[doc = include_str!("../../docs/api/notification/notification_provider.doc/permission.md")]
    #[must_use]
    fn permission(&self) -> SupportLevel;

    /// Posts `notification` for presentation and returns the honest outcome
    /// (spec §6.12).
    fn post(&mut self, notification: Notification) -> NotificationResult;

    /// Sets the app-icon badge to `count`, or clears it when `None` (spec §6.12).
    fn set_badge(&mut self, count: Option<u32>);
}

#[cfg(test)]
mod tests {
    use super::super::NullNotificationProvider;
    use super::*;

    #[test]
    fn notification_provider_is_object_safe() {
        let _: Box<dyn NotificationProvider> = Box::new(NullNotificationProvider);
    }
}
