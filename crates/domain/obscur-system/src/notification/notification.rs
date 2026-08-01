#![doc = include_str!("notification.doc/_module.md")]

use crate::notification::NotificationAction;

#[doc = include_str!("notification.doc/Notification.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    #[doc = include_str!("notification.doc/title.md")]
    title: Box<str>,
    #[doc = include_str!("notification.doc/body.md")]
    body: Box<str>,
    #[doc = include_str!("notification.doc/badge.md")]
    badge: Option<u32>,
    #[doc = include_str!("notification.doc/progress.md")]
    progress: Option<f32>,
    #[doc = include_str!("notification.doc/actions.md")]
    actions: Vec<NotificationAction>,
    #[doc = include_str!("notification.doc/request_attention.md")]
    request_attention: bool,
}

impl Notification {
    #[doc = include_str!("notification.doc/new.md")]
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

    #[doc = include_str!("notification.doc/with_badge.md")]
    #[must_use]
    pub fn with_badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    #[doc = include_str!("notification.doc/with_progress.md")]
    #[must_use]
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    #[doc = include_str!("notification.doc/with_action.md")]
    #[must_use]
    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }

    #[doc = include_str!("notification.doc/requesting_attention.md")]
    #[must_use]
    pub fn requesting_attention(mut self, request: bool) -> Self {
        self.request_attention = request;
        self
    }

    #[doc = include_str!("notification.doc/title_2.md")]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[doc = include_str!("notification.doc/body_2.md")]
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    #[doc = include_str!("notification.doc/badge_2.md")]
    #[must_use]
    pub const fn badge(&self) -> Option<u32> {
        self.badge
    }

    #[doc = include_str!("notification.doc/progress_2.md")]
    #[must_use]
    pub const fn progress(&self) -> Option<f32> {
        self.progress
    }

    #[doc = include_str!("notification.doc/actions_2.md")]
    #[must_use]
    pub fn actions(&self) -> &[NotificationAction] {
        &self.actions
    }

    #[doc = include_str!("notification.doc/request_attention_2.md")]
    #[must_use]
    pub const fn request_attention(&self) -> bool {
        self.request_attention
    }

    #[doc = include_str!("notification.doc/has_actions.md")]
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
