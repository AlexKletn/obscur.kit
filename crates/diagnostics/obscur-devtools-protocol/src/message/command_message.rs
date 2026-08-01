#![doc = include_str!("command_message.doc/_module.md")]

use super::{Inspectable, PermissionScope, SubscriptionId, Topic};

#[doc = include_str!("command_message.doc/CommandId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandId(pub u64);

impl CommandId {
    #[doc = include_str!("command_message.doc/new.md")]
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("command_message.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("command_message.doc/ControlAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    #[doc = include_str!("command_message.doc/Pause.md")]
    Pause,
    #[doc = include_str!("command_message.doc/Resume.md")]
    Resume,
    #[doc = include_str!("command_message.doc/StepFrame.md")]
    StepFrame,
    #[doc = include_str!("command_message.doc/SetInspectMode.md")]
    SetInspectMode(bool),
}

#[doc = include_str!("command_message.doc/InjectedEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectedEvent {
    #[doc = include_str!("command_message.doc/PointerDown.md")]
    PointerDown {
        #[doc = include_str!("command_message.doc/x.md")]
        x: i32,
        #[doc = include_str!("command_message.doc/y.md")]
        y: i32,
    },
    #[doc = include_str!("command_message.doc/PointerUp.md")]
    PointerUp {
        #[doc = include_str!("command_message.doc/x_2.md")]
        x: i32,
        #[doc = include_str!("command_message.doc/y_2.md")]
        y: i32,
    },
    #[doc = include_str!("command_message.doc/KeyDown.md")]
    KeyDown {
        #[doc = include_str!("command_message.doc/key.md")]
        key: u32,
    },
    #[doc = include_str!("command_message.doc/KeyUp.md")]
    KeyUp {
        #[doc = include_str!("command_message.doc/key_2.md")]
        key: u32,
    },
}

#[doc = include_str!("command_message.doc/CommandKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    #[doc = include_str!("command_message.doc/Inspect.md")]
    Inspect(Inspectable),
    #[doc = include_str!("command_message.doc/Subscribe.md")]
    Subscribe(Topic),
    #[doc = include_str!("command_message.doc/Unsubscribe.md")]
    Unsubscribe(SubscriptionId),
    #[doc = include_str!("command_message.doc/Control.md")]
    Control(ControlAction),
    #[doc = include_str!("command_message.doc/Inject.md")]
    Inject(InjectedEvent),
}

impl CommandKind {
    #[doc = include_str!("command_message.doc/required_scope.md")]
    #[must_use]
    pub const fn required_scope(&self) -> PermissionScope {
        match self {
            Self::Inspect(_) => PermissionScope::Read,
            Self::Subscribe(_) | Self::Unsubscribe(_) => PermissionScope::Subscribe,
            Self::Control(_) => PermissionScope::Control,
            Self::Inject(_) => PermissionScope::Inject,
        }
    }

    #[doc = include_str!("command_message.doc/is_dangerous.md")]
    #[must_use]
    pub const fn is_dangerous(&self) -> bool {
        matches!(self, Self::Control(_) | Self::Inject(_))
    }
}

#[doc = include_str!("command_message.doc/CommandMessage.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandMessage {
    #[doc = include_str!("command_message.doc/id.md")]
    pub id: CommandId,
    #[doc = include_str!("command_message.doc/kind.md")]
    pub kind: CommandKind,
}

impl CommandMessage {
    #[doc = include_str!("command_message.doc/new_2.md")]
    #[must_use]
    pub const fn new(id: CommandId, kind: CommandKind) -> Self {
        Self { id, kind }
    }

    #[doc = include_str!("command_message.doc/required_scope_2.md")]
    #[must_use]
    pub const fn required_scope(&self) -> PermissionScope {
        self.kind.required_scope()
    }

    #[doc = include_str!("command_message.doc/is_dangerous_2.md")]
    #[must_use]
    pub const fn is_dangerous(&self) -> bool {
        self.kind.is_dangerous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_scope_matches_kind() {
        assert_eq!(
            CommandKind::Inspect(Inspectable::Tree).required_scope(),
            PermissionScope::Read
        );
        assert_eq!(
            CommandKind::Subscribe(Topic::Frames).required_scope(),
            PermissionScope::Subscribe
        );
        assert_eq!(
            CommandKind::Unsubscribe(SubscriptionId::new(1)).required_scope(),
            PermissionScope::Subscribe
        );
        assert_eq!(
            CommandKind::Control(ControlAction::Pause).required_scope(),
            PermissionScope::Control
        );
        assert_eq!(
            CommandKind::Inject(InjectedEvent::KeyDown { key: 65 }).required_scope(),
            PermissionScope::Inject
        );
    }

    #[test]
    fn only_control_and_inject_are_dangerous() {
        assert!(CommandKind::Control(ControlAction::StepFrame).is_dangerous());
        assert!(CommandKind::Inject(InjectedEvent::PointerDown { x: 1, y: 2 }).is_dangerous());
        assert!(!CommandKind::Inspect(Inspectable::Styles).is_dangerous());
        assert!(!CommandKind::Subscribe(Topic::Diagnostics).is_dangerous());
        assert!(!CommandKind::Unsubscribe(SubscriptionId::new(2)).is_dangerous());
    }

    #[test]
    fn message_delegates_to_kind() {
        let msg = CommandMessage::new(
            CommandId::new(7),
            CommandKind::Control(ControlAction::SetInspectMode(true)),
        );
        assert_eq!(msg.id.raw(), 7);
        assert_eq!(msg.required_scope(), PermissionScope::Control);
        assert!(msg.is_dangerous());
    }

    #[test]
    fn command_id_round_trip() {
        assert_eq!(CommandId::new(42).raw(), 42);
        assert_eq!(CommandId::default(), CommandId::new(0));
    }
}
