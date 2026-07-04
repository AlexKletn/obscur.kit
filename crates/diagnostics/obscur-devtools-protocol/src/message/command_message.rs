//! Client→server command envelope and its capability-scoped kinds (§7.6/§7.7).

use super::{Inspectable, PermissionScope, SubscriptionId, Topic};

/// A client-assigned identifier correlating a command with its response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandId(pub u64);

impl CommandId {
    /// Creates a command id from a raw value.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A typed control action affecting the running application (spec §7.6).
///
/// Control actions are a small, closed enum: DevTools may steer the runtime but
/// can never supply arbitrary code (spec §7.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    /// Pause the runtime (freeze frame production).
    Pause,
    /// Resume a paused runtime.
    Resume,
    /// Advance exactly one frame while paused.
    StepFrame,
    /// Enable (`true`) or disable (`false`) pick/inspect mode.
    SetInspectMode(bool),
}

/// A typed, synthetic event injected into the runtime (spec §7.6).
///
/// Injection is restricted to a closed set of typed descriptors — never
/// arbitrary code or memory writes (spec §7.7). Coordinates are logical
/// integers; key events reference a stable key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectedEvent {
    /// A synthetic pointer press at logical coordinates `(x, y)`.
    PointerDown {
        /// Logical x coordinate.
        x: i32,
        /// Logical y coordinate.
        y: i32,
    },
    /// A synthetic pointer release at logical coordinates `(x, y)`.
    PointerUp {
        /// Logical x coordinate.
        x: i32,
        /// Logical y coordinate.
        y: i32,
    },
    /// A synthetic key press identified by a stable key code.
    KeyDown {
        /// Stable key code.
        key: u32,
    },
    /// A synthetic key release identified by a stable key code.
    KeyUp {
        /// Stable key code.
        key: u32,
    },
}

/// The typed body of a [`CommandMessage`].
///
/// Each variant maps to exactly one [`PermissionScope`] via
/// [`required_scope`](CommandKind::required_scope), the single source of truth
/// the server uses to authorize a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    /// Request a one-shot snapshot of an inspectable target (needs `Read`).
    Inspect(Inspectable),
    /// Subscribe to an event stream (needs `Subscribe`).
    Subscribe(Topic),
    /// Cancel an existing subscription (needs `Subscribe`).
    Unsubscribe(SubscriptionId),
    /// Perform a control action (needs `Control`; dangerous → audited).
    Control(ControlAction),
    /// Inject a synthetic event (needs `Inject`; dangerous → audited).
    Inject(InjectedEvent),
}

impl CommandKind {
    /// Returns the permission scope required to execute this command.
    ///
    /// This is the authoritative capability check the server performs before
    /// dispatch (spec §7.7).
    #[must_use]
    pub const fn required_scope(&self) -> PermissionScope {
        match self {
            Self::Inspect(_) => PermissionScope::Read,
            Self::Subscribe(_) | Self::Unsubscribe(_) => PermissionScope::Subscribe,
            Self::Control(_) => PermissionScope::Control,
            Self::Inject(_) => PermissionScope::Inject,
        }
    }

    /// Returns whether this command is a dangerous operation that must be
    /// audited (spec §7.7).
    ///
    /// True only for control and event injection.
    #[must_use]
    pub const fn is_dangerous(&self) -> bool {
        matches!(self, Self::Control(_) | Self::Inject(_))
    }
}

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
