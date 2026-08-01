#![doc = include_str!("notification_action.doc/_module.md")]

#[doc = include_str!("notification_action.doc/NotificationAction.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationAction {
    #[doc = include_str!("notification_action.doc/id.md")]
    id: Box<str>,
    #[doc = include_str!("notification_action.doc/label.md")]
    label: Box<str>,
}

impl NotificationAction {
    #[doc = include_str!("notification_action.doc/new.md")]
    #[must_use]
    pub fn new(id: impl Into<Box<str>>, label: impl Into<Box<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    #[doc = include_str!("notification_action.doc/id_2.md")]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[doc = include_str!("notification_action.doc/label_2.md")]
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
