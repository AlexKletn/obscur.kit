//! The outcome of dispatching one keyboard event: [`KeyDispatch`].

use obscur_identity::{InternedSymbol, NodeId};

/// The result of routing a keyboard event through
/// [`UiRuntime::handle_key`](super::UiRuntime::handle_key).
///
/// It is the keyboard analogue of
/// [`PointerDispatch`](super::PointerDispatch): it reports the focused
/// [`target`](Self::target) the key was routed to (if any), the
/// [`command`](Self::command) symbol the runtime's
/// [`CommandRouter`](obscur_interaction::command::CommandRouter) resolved the key
/// press to (if a bound shortcut matched — the command was also *dispatched*, so
/// its registered handler ran), and whether the key was
/// [`handled`](Self::is_handled) — i.e. reached a focused node.
///
/// When no node holds focus the key routes nowhere: `target` is `None`,
/// `command` is `None`, and `is_handled()` is `false` — the runtime's state is
/// unchanged and no frame is requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyDispatch {
    /// The focused node the key was routed to, or `None` when nothing is focused.
    target: Option<NodeId>,
    /// The command symbol a bound shortcut resolved (and dispatched) the key
    /// press to, if any.
    command: Option<InternedSymbol>,
}

impl KeyDispatch {
    /// Constructs a dispatch outcome from the routed `target` and the resolved
    /// `command` symbol.
    #[must_use]
    pub const fn new(target: Option<NodeId>, command: Option<InternedSymbol>) -> Self {
        Self { target, command }
    }

    /// The empty outcome: the key reached no focused node and resolved no
    /// command.
    #[must_use]
    pub const fn unhandled() -> Self {
        Self {
            target: None,
            command: None,
        }
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
