use super::{
    ControlAction, InjectedEvent, Inspectable, PermissionScope, StableSelector, SubscriptionId,
    Topic,
};

#[doc = include_str!("../../docs/api/message/command_message.doc/CommandKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    /// Request a one-shot snapshot of an inspectable target (needs `Read`).
    Inspect(Inspectable),
    /// Queries visible accessibility nodes by role and optional exact name.
    SelectSemantic {
        /// Semantic role label.
        role: Box<str>,
        /// Exact accessible name, when specified.
        name: Option<Box<str>>,
    },
    /// Finds retained nodes by a stable identifier.
    SelectStableId(StableSelector),
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
    #[doc = include_str!("../../docs/api/message/command_message.doc/required_scope.md")]
    #[must_use]
    pub const fn required_scope(&self) -> PermissionScope {
        match self {
            Self::Inspect(_) | Self::SelectSemantic { .. } | Self::SelectStableId(_) => {
                PermissionScope::Read
            }
            Self::Subscribe(_) | Self::Unsubscribe(_) => PermissionScope::Subscribe,
            Self::Control(_) => PermissionScope::Control,
            Self::Inject(_) => PermissionScope::Inject,
        }
    }

    #[doc = include_str!("../../docs/api/message/command_message.doc/is_dangerous.md")]
    #[must_use]
    pub const fn is_dangerous(&self) -> bool {
        matches!(self, Self::Control(_) | Self::Inject(_))
    }
}
