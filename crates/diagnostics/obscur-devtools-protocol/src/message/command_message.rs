//! Client→server command envelope and its capability-scoped kinds (§7.6/§7.7).

use super::{CommandId, CommandKind, PermissionScope};

/// A single client→server command with a correlation id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandMessage {
    /// Client-assigned id used to correlate the response/snapshot.
    pub id: CommandId,
    /// The command body.
    pub kind: CommandKind,
}

impl CommandMessage {
    /// Creates a command message.
    #[must_use]
    pub const fn new(id: CommandId, kind: CommandKind) -> Self {
        Self { id, kind }
    }

    /// The permission scope required to execute this command.
    #[must_use]
    pub const fn required_scope(&self) -> PermissionScope {
        self.kind.required_scope()
    }

    /// Whether this command is a dangerous, audited operation.
    #[must_use]
    pub const fn is_dangerous(&self) -> bool {
        self.kind.is_dangerous()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ControlAction, InjectedEvent, Inspectable, SubscriptionId, Topic};
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
