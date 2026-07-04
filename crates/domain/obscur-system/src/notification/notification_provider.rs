//! The replaceable port for presenting system notifications (spec §6.12).
//!
//! [`NotificationProvider`] is the hexagonal port through which the domain asks
//! the host to [`post`](NotificationProvider::post) a
//! [`Notification`] and to manage the app-icon badge. Permission and host policy
//! are **explicit** (spec §6.12): a provider reports its honest
//! [`permission`](NotificationProvider::permission) as a [`SupportLevel`], and a
//! refusal is the first-class [`NotificationResult::Denied`] outcome rather than a
//! panic. The colocated [`NullNotificationProvider`] and
//! [`RecordingNotificationProvider`] are reference implementations of that
//! contract.

use obscur_capability::SupportLevel;

use crate::notification::Notification;

/// The outcome of posting a [`Notification`] through a [`NotificationProvider`]
/// (spec §6.12).
///
/// Because permission is explicit (spec §6.12), [`Denied`](Self::Denied) is a
/// first-class outcome distinct from [`Unsupported`](Self::Unsupported): the host
/// is capable of notifications but policy refused this one, versus the host offers
/// no notification facility at all.
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

/// The replaceable port for presenting system notifications (spec §6.12).
///
/// An adapter implements this trait once per platform and the domain holds it as
/// `Box<dyn NotificationProvider>`. The trait is deliberately **object-safe** — no
/// generic or `Self`-returning methods — so providers can be stored
/// polymorphically.
///
/// Contract: a provider reports its honest [`permission`](Self::permission) and
/// never panics — a refusal is [`NotificationResult::Denied`] and an absent
/// facility is [`NotificationResult::Unsupported`] (spec §6.12).
pub trait NotificationProvider {
    /// Reports the host's honest notification permission / capability level
    /// (spec §6.12).
    ///
    /// Permission is explicit: [`SupportLevel::Unavailable`] means the host will
    /// present nothing, regardless of the [`Notification`] posted.
    #[must_use]
    fn permission(&self) -> SupportLevel;

    /// Posts `notification` for presentation and returns the honest outcome
    /// (spec §6.12).
    fn post(&mut self, notification: Notification) -> NotificationResult;

    /// Sets the app-icon badge to `count`, or clears it when `None` (spec §6.12).
    fn set_badge(&mut self, count: Option<u32>);
}

/// A reference, no-op [`NotificationProvider`] that presents nothing
/// (spec §6.12).
///
/// `NullNotificationProvider` honestly reports [`SupportLevel::Unavailable`]:
/// [`post`](Self::post) returns [`NotificationResult::Unsupported`] and
/// [`set_badge`](Self::set_badge) is a no-op. Its answers are correct behavior — a
/// documented reference for the honest-capability contract — not a stub waiting to
/// be filled in.
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

/// A deterministic, recording [`NotificationProvider`] for tests (spec §6.12).
///
/// `RecordingNotificationProvider` reports [`SupportLevel::Native`] and accepts
/// everything: [`post`](Self::post) pushes the notification onto
/// [`posted`](Self::posted) and returns [`NotificationResult::Delivered`], and
/// [`set_badge`](Self::set_badge) stores the value in [`badge`](Self::badge).
/// Tests inspect the recorded state to assert what the domain attempted, without
/// touching a real notification service.
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
