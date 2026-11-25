//! The outcome of dispatching one keyboard event: [`KeyDispatch`].

use obscur_identity::{InternedSymbol, NodeId};

#[doc = include_str!("../../docs/api/runtime/key_dispatch.doc/KeyDispatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyDispatch {
    /// The focused node the key was routed to, or `None` when nothing is focused.
    target: Option<NodeId>,
    /// The resolved command for a shortcut or keyboard activation, if any.
    command: Option<InternedSymbol>,
    default_prevented: bool,
}

impl KeyDispatch {
    /// Constructs a dispatch outcome from the routed `target` and the resolved
    /// `command` symbol.
    #[must_use]
    pub const fn new(target: Option<NodeId>, command: Option<InternedSymbol>) -> Self {
        Self {
            target,
            command,
            default_prevented: false,
        }
    }

    /// The empty outcome: the key reached no focused node and resolved no
    /// command.
    #[must_use]
    pub const fn unhandled() -> Self {
        Self::new(None, None)
    }

    pub(crate) const fn with_default_prevented(mut self, prevented: bool) -> Self {
        self.default_prevented = prevented;
        self
    }

    /// Whether a raw keyboard handler called `prevent_default`.
    #[must_use]
    pub const fn default_prevented(&self) -> bool {
        self.default_prevented
    }

    /// The focused node the key was routed to, if any.
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        self.target
    }

    /// The command symbol the key press resolved to, if a bound shortcut matched.
    #[must_use]
    pub const fn command(&self) -> Option<InternedSymbol> {
        self.command
    }

    /// Whether the key reached a focused node (was routed somewhere).
    #[must_use]
    pub const fn is_handled(&self) -> bool {
        self.target.is_some()
    }

    /// Whether the key press resolved to a bound command.
    #[must_use]
    pub const fn resolved_command(&self) -> bool {
        self.command.is_some()
    }
}
