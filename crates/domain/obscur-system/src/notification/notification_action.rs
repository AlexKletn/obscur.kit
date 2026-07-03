//! An actionable button attached to a notification (spec §6.12 — notifications).

/// An actionable button on a [`Notification`](crate::notification::Notification)
/// (spec §6.12 — notification actions).
///
/// An action is a pure value: a stable [`id`](Self::id) the host echoes back when
/// the user clicks the button, paired with the [`label`](Self::label) shown on it.
/// The domain matches the echoed id against the actions it attached; the display
/// text is never used for identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationAction {
    /// Stable action identity echoed back to the app when the button is clicked.
    id: Box<str>,
    /// Human-readable text displayed on the button.
    label: Box<str>,
}

impl NotificationAction {
    /// Creates an action with the stable `id` and display `label` (spec §6.12).
    #[must_use]
    pub fn new(id: impl Into<Box<str>>, label: impl Into<Box<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    /// Returns the stable action id echoed back when the button is clicked
    /// (spec §6.12).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display text shown on the button (spec §6.12).
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_id_and_label() {
        let action = NotificationAction::new("reply", "Reply");
        assert_eq!(action.id(), "reply");
        assert_eq!(action.label(), "Reply");
    }

    #[test]
    fn identity_is_by_all_fields() {
        let a = NotificationAction::new("reply", "Reply");
        let b = NotificationAction::new("reply", "Reply");
        assert_eq!(a, b);
        assert_ne!(a, NotificationAction::new("reply", "Respond"));
        assert_ne!(a, NotificationAction::new("dismiss", "Reply"));
    }
}
