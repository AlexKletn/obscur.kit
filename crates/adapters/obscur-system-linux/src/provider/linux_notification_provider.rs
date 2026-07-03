//! A Linux [`NotificationProvider`] that posts via `notify-send` (spec §6.12).
//!
//! [`LinuxNotificationProvider`] implements the pure
//! [`NotificationProvider`](obscur_system::notification::NotificationProvider)
//! port by shelling out to `notify-send` with [`std::process::Command`] and no
//! external crates. A [`Notification`](obscur_system::notification::Notification)
//! maps to a `notify-send` argv (see [`build_args`]); the provider spawns the tool
//! fire-and-forget. `notify-send` failing to spawn usually means no notification
//! daemon is running, which maps to [`NotificationResult::Denied`].
//!
//! Linux has no standard per-application icon badge, so [`Self::set_badge`] only
//! records the value — surfacing it (for example on the Unity launcher) would need
//! a D-Bus round-trip and is a future refinement (spec §2.7 target-specific
//! providers).

use std::process::Command;

use obscur_capability::SupportLevel;
use obscur_system::notification::{Notification, NotificationProvider, NotificationResult};

/// A Linux [`NotificationProvider`] backed by `notify-send` (spec §6.12).
///
/// The provider holds the tool command name (defaulting to `"notify-send"`) and
/// the last badge value set through [`Self::set_badge`]. [`Self::permission`]
/// scans `PATH` for the tool without executing it — so it never pops a
/// notification — and [`Self::post`] spawns the tool fire-and-forget.
#[derive(Debug, Clone)]
pub struct LinuxNotificationProvider {
    /// The notification tool to invoke, e.g. `"notify-send"`.
    tool: Box<str>,
    /// The last app-icon badge count set; recorded but not surfaced on Linux.
    badge: Option<u32>,
}

impl LinuxNotificationProvider {
    /// Creates a provider that posts notifications with `notify-send`
    /// (spec §6.12).
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool: Box::from("notify-send"),
            badge: None,
        }
    }

    /// Creates a provider that posts notifications with a custom `tool` command
    /// (spec §6.12).
    #[must_use]
    pub fn with_tool(tool: impl Into<Box<str>>) -> Self {
        Self {
            tool: tool.into(),
            badge: None,
        }
    }

    /// Returns the notification tool this provider invokes (spec §6.12).
    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    /// Returns the last recorded app-icon badge value (spec §6.12).
    ///
    /// Linux has no standard app-badge, so this is recorded state only — it is not
    /// shown to the user.
    #[must_use]
    pub const fn badge(&self) -> Option<u32> {
        self.badge
    }
}

impl Default for LinuxNotificationProvider {
    /// The default provider posts notifications with `notify-send`.
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationProvider for LinuxNotificationProvider {
    fn permission(&self) -> SupportLevel {
        if tool_available(&self.tool) {
            SupportLevel::Native
        } else {
            SupportLevel::Unavailable
        }
    }

    fn post(&mut self, notification: Notification) -> NotificationResult {
        if !tool_available(&self.tool) {
            // No notify-send on PATH: no notification facility.
            return NotificationResult::Unsupported;
        }
        let args = build_args(&notification);
        match Command::new(&*self.tool).args(&args).spawn() {
            // Fire-and-forget: the daemon presents the notification asynchronously.
            Ok(_child) => NotificationResult::Delivered,
            // A spawn failure usually means no notification daemon is running:
            // the host is capable but this post was effectively refused.
            Err(_io) => NotificationResult::Denied,
        }
    }

    fn set_badge(&mut self, count: Option<u32>) {
        // Recorded but not surfaced: Linux has no standard app-icon badge. A
        // Unity-launcher badge would need D-Bus, a future refinement.
        self.badge = count;
    }
}

/// Builds the `notify-send` argv for `notification`, excluding the tool name
/// (spec §6.12).
///
/// This is the pure notification→argv mapping, factored out so the flag choices
/// can be unit-tested without spawning `notify-send`. When the notification
/// [requests attention](Notification::request_attention) the argv leads with
/// `--urgency=critical`; the title and body then follow as positional arguments.
#[must_use]
fn build_args(notification: &Notification) -> Vec<String> {
    let mut args = Vec::new();
    if notification.request_attention() {
        args.push("--urgency=critical".to_string());
    }
    args.push(notification.title().to_string());
    args.push(notification.body().to_string());
    args
}

/// Returns whether an executable named `cmd` exists on `PATH` (spec §2.7).
///
/// The probe scans the `PATH` environment variable, splitting on `:`, and tests
/// each `dir/cmd` with [`std::fs::metadata`]. It deliberately does **not** execute
/// the tool, so capability queries are deterministic and never pop a notification.
fn tool_available(cmd: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    path.to_string_lossy()
        .split(':')
        .filter(|dir| !dir.is_empty())
        .any(|dir| std::fs::metadata(std::path::Path::new(dir).join(cmd)).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_is_notify_send() {
        assert_eq!(LinuxNotificationProvider::new().tool(), "notify-send");
        assert_eq!(LinuxNotificationProvider::default().tool(), "notify-send");
    }

    #[test]
    fn missing_tool_is_unavailable_and_unsupported() {
        let mut provider =
            LinuxNotificationProvider::with_tool("definitely-not-a-real-tool-xyz123");
        assert_eq!(provider.permission(), SupportLevel::Unavailable);
        assert_eq!(
            provider.post(Notification::new("t", "b")),
            NotificationResult::Unsupported
        );
    }

    #[test]
    fn set_badge_records_without_surfacing() {
        let mut provider = LinuxNotificationProvider::new();
        assert_eq!(provider.badge(), None);
        provider.set_badge(Some(4));
        assert_eq!(provider.badge(), Some(4));
        provider.set_badge(None);
        assert_eq!(provider.badge(), None);
    }

    #[test]
    fn build_args_plain_notification_is_title_body() {
        assert_eq!(
            build_args(&Notification::new("Sync", "Done")),
            ["Sync", "Done"]
        );
    }

    #[test]
    fn build_args_attention_leads_with_critical_urgency() {
        assert_eq!(
            build_args(&Notification::new("Alert", "Now").requesting_attention(true)),
            ["--urgency=critical", "Alert", "Now"]
        );
    }
}
