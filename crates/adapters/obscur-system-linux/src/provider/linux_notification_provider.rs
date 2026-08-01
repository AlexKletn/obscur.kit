#![doc = include_str!("linux_notification_provider.doc/_module.md")]

use std::process::Command;

use obscur_capability::SupportLevel;
use obscur_system::notification::{Notification, NotificationProvider, NotificationResult};

#[doc = include_str!("linux_notification_provider.doc/LinuxNotificationProvider.md")]
#[derive(Debug, Clone)]
pub struct LinuxNotificationProvider {
    #[doc = include_str!("linux_notification_provider.doc/tool.md")]
    tool: Box<str>,
    #[doc = include_str!("linux_notification_provider.doc/badge.md")]
    badge: Option<u32>,
}

impl LinuxNotificationProvider {
    #[doc = include_str!("linux_notification_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool: Box::from("notify-send"),
            badge: None,
        }
    }

    #[doc = include_str!("linux_notification_provider.doc/with_tool.md")]
    #[must_use]
    pub fn with_tool(tool: impl Into<Box<str>>) -> Self {
        Self {
            tool: tool.into(),
            badge: None,
        }
    }

    #[doc = include_str!("linux_notification_provider.doc/tool_2.md")]
    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    #[doc = include_str!("linux_notification_provider.doc/badge_2.md")]
    #[must_use]
    pub const fn badge(&self) -> Option<u32> {
        self.badge
    }
}

impl Default for LinuxNotificationProvider {
    #[doc = include_str!("linux_notification_provider.doc/default.md")]
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

#[doc = include_str!("linux_notification_provider.doc/build_args.md")]
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

#[doc = include_str!("linux_notification_provider.doc/tool_available.md")]
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
