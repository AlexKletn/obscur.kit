//! A web [`NotificationProvider`] backed by the browser `Notification` API
//! (spec §6.12).
//!
//! [`WebNotificationProvider`] implements the pure
//! [`NotificationProvider`](obscur_system::notification::NotificationProvider)
//! port over `web-sys`'s [`web_sys::Notification`]. Because the port is
//! synchronous, this provider only *reads* the current permission and posts when
//! it is already `Granted`: requesting permission is an asynchronous browser call
//! (`Notification.requestPermission()`) that must be driven by the application
//! before using this provider, and is out of scope for this sync port.
//!
//! All browser calls are gated behind `#[cfg(target_arch = "wasm32")]`; on any
//! other target the provider reports [`SupportLevel::Unavailable`] and
//! [`NotificationResult::Unsupported`], keeping the host build honest.

use obscur_capability::SupportLevel;
use obscur_system::notification::{Notification, NotificationProvider, NotificationResult};

/// A web [`NotificationProvider`] backed by the browser `Notification` API
/// (spec §6.12).
///
/// The provider records the last app-icon badge value set through
/// [`Self::set_badge`]. Surfacing it via the browser Badging API
/// (`navigator.setAppBadge`) is a documented future refinement; for now the badge
/// is stored but not displayed. [`Self::permission`] reads the browser's current
/// `Notification.permission`, and [`Self::post`] presents a notification only when
/// permission is already granted.
#[derive(Debug, Clone, Default)]
pub struct WebNotificationProvider {
    /// The last app-icon badge count set; recorded but not surfaced.
    badge: Option<u32>,
}

impl WebNotificationProvider {
    /// Creates a web notification provider with no badge set (spec §6.12).
    #[must_use]
    pub const fn new() -> Self {
        Self { badge: None }
    }

    /// Returns the last recorded app-icon badge value (spec §6.12).
    ///
    /// The value is recorded state only; presenting it via the Badging API is a
    /// future refinement.
    #[must_use]
    pub const fn badge(&self) -> Option<u32> {
        self.badge
    }
}

impl NotificationProvider for WebNotificationProvider {
    fn permission(&self) -> SupportLevel {
        #[cfg(target_arch = "wasm32")]
        {
            // Map the browser's tri-state permission onto the honest support level:
            // granted → native, default → limited (an async request is still
            // required), denied → unavailable.
            match web_sys::Notification::permission() {
                web_sys::NotificationPermission::Granted => SupportLevel::Native,
                web_sys::NotificationPermission::Denied => SupportLevel::Unavailable,
                // `Default` and any future variant: capable, but needs a request.
                _ => SupportLevel::Limited,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SupportLevel::Unavailable
        }
    }

    fn post(&mut self, notification: Notification) -> NotificationResult {
        #[cfg(target_arch = "wasm32")]
        {
            // Only post when permission is already granted. Requesting it is an
            // async browser call the app must make ahead of time — out of this
            // sync port.
            if web_sys::Notification::permission() != web_sys::NotificationPermission::Granted {
                return NotificationResult::Denied;
            }
            let opts = web_sys::NotificationOptions::new();
            opts.set_body(notification.body());
            match web_sys::Notification::new_with_options(notification.title(), &opts) {
                Ok(_notification) => NotificationResult::Delivered,
                Err(_err) => NotificationResult::Denied,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = notification;
            NotificationResult::Unsupported
        }
    }

    fn set_badge(&mut self, count: Option<u32>) {
        // Recorded but not surfaced: the browser Badging API
        // (`navigator.setAppBadge`) is a future refinement, so this only stores the
        // value on both wasm and host targets.
        self.badge = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_is_unavailable_on_host_target() {
        let provider = WebNotificationProvider::new();
        assert_eq!(provider.permission(), SupportLevel::Unavailable);
    }

    #[test]
    fn post_is_unsupported_on_host_target() {
        let mut provider = WebNotificationProvider::new();
        assert_eq!(
            provider.post(Notification::new("Sync", "Done")),
            NotificationResult::Unsupported
        );
    }

    #[test]
    fn set_badge_records_without_surfacing() {
        let mut provider = WebNotificationProvider::new();
        assert_eq!(provider.badge(), None);
        provider.set_badge(Some(4));
        assert_eq!(provider.badge(), Some(4));
        provider.set_badge(None);
        assert_eq!(provider.badge(), None);
    }
}
