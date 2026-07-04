//! A system notification the app asks the host to present (spec §6.12).

use crate::notification::NotificationAction;

/// A system notification (spec §6.12 — notifications).
///
/// A notification is a pure value describing what the app wants shown: a
/// [`title`](Self::title) and [`body`](Self::body), an optional app-icon
/// [`badge`](Self::badge) count, optional determinate [`progress`](Self::progress),
/// a set of actionable [`actions`](Self::actions), and whether it should
/// [request attention](Self::request_attention) (bounce/flash). Whether any of
/// this is honored is the host's decision — permission and presentation policy are
/// explicit and live behind the
/// [`NotificationProvider`](crate::notification::NotificationProvider) port
/// (spec §6.12).
///
/// Build one with [`new`](Self::new) and layer on optional facets with the fluent
/// `with_*` / `requesting_attention` methods.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    /// The notification's headline text.
    title: Box<str>,
    /// The notification's body / detail text.
    body: Box<str>,
    /// Optional app-icon badge count to display alongside this notification.
    badge: Option<u32>,
    /// Optional determinate progress in `0.0..=1.0`; `None` is no/indeterminate
    /// progress.
    progress: Option<f32>,
    /// Actionable buttons offered on the notification.
    actions: Vec<NotificationAction>,
    /// Whether to request user attention (bounce the dock icon / flash the task
    /// entry).
    request_attention: bool,
}

impl Notification {
    /// Creates a notification with `title` and `body`; badge, progress, and
    /// actions start empty and attention is not requested (spec §6.12).
    #[must_use]
    pub fn new(title: impl Into<Box<str>>, body: impl Into<Box<str>>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            badge: None,
            progress: None,
            actions: Vec::new(),
            request_attention: false,
        }
    }

    /// Returns this notification with the app-icon badge `count` set (spec §6.12).
    #[must_use]
    pub fn with_badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    /// Returns this notification with determinate `progress`, clamped into
    /// `0.0..=1.0` (spec §6.12).
    #[must_use]
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    /// Returns this notification with `action` appended to its actions
    /// (spec §6.12).
    #[must_use]
    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Returns this notification with the request-attention flag set to
    /// `request` (spec §6.12).
    #[must_use]
    pub fn requesting_attention(mut self, request: bool) -> Self {
        self.request_attention = request;
        self
    }

    /// Returns the headline text (spec §6.12).
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the body / detail text (spec §6.12).
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the app-icon badge count, if any (spec §6.12).
    #[must_use]
    pub const fn badge(&self) -> Option<u32> {
        self.badge
    }

    /// Returns the determinate progress in `0.0..=1.0`, if any (spec §6.12).
    #[must_use]
    pub const fn progress(&self) -> Option<f32> {
        self.progress
    }

    /// Returns the actionable buttons offered on this notification (spec §6.12).
    #[must_use]
    pub fn actions(&self) -> &[NotificationAction] {
        &self.actions
    }

    /// Returns whether this notification requests user attention (spec §6.12).
    #[must_use]
    pub const fn request_attention(&self) -> bool {
        self.request_attention
    }

    /// Returns whether this notification carries any actions (spec §6.12).
    #[must_use]
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_leaves_optionals_empty() {
        let note = Notification::new("Title", "Body");
        assert_eq!(note.title(), "Title");
        assert_eq!(note.body(), "Body");
        assert_eq!(note.badge(), None);
        assert_eq!(note.progress(), None);
        assert!(note.actions().is_empty());
        assert!(!note.has_actions());
        assert!(!note.request_attention());
    }

    #[test]
    fn builder_layers_facets() {
        let note = Notification::new("Sync", "In progress")
            .with_badge(3)
            .with_progress(0.5)
            .with_action(NotificationAction::new("cancel", "Cancel"))
            .requesting_attention(true);
        assert_eq!(note.badge(), Some(3));
        assert_eq!(note.progress(), Some(0.5));
        assert!(note.has_actions());
        assert_eq!(note.actions().len(), 1);
        assert_eq!(note.actions()[0].id(), "cancel");
        assert!(note.request_attention());
    }

    #[test]
    fn progress_is_clamped() {
        assert_eq!(
            Notification::new("t", "b").with_progress(1.5).progress(),
            Some(1.0)
        );
        assert_eq!(
            Notification::new("t", "b").with_progress(-0.5).progress(),
            Some(0.0)
        );
        assert_eq!(
            Notification::new("t", "b").with_progress(0.25).progress(),
            Some(0.25)
        );
    }

    #[test]
    fn with_action_appends_in_order() {
        let note = Notification::new("t", "b")
            .with_action(NotificationAction::new("a", "A"))
            .with_action(NotificationAction::new("b", "B"));
        assert_eq!(note.actions().len(), 2);
        assert_eq!(note.actions()[0].id(), "a");
        assert_eq!(note.actions()[1].id(), "b");
    }
}
