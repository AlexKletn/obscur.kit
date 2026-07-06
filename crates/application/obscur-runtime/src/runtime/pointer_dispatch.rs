//! The outcome of dispatching one pointer event: [`PointerDispatch`].

use obscur_identity::{InternedSymbol, NodeId};

/// The result of routing a pointer event through
/// [`UiRuntime::handle_pointer`](super::UiRuntime::handle_pointer).
///
/// It reports the node the pointer resolved to (the topmost hit, if any) and
/// whether this event actually moved the hover chain or the focus target —
/// the two signals a host uses to decide whether a restyle/repaint is warranted.
/// When both `hover_changed` and `focus_changed` are `false`, the runtime's
/// interaction state is byte-for-byte unchanged and no frame was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointerDispatch {
    /// The topmost node under the pointer after hit-testing, or `None` when the
    /// pointer is over no indexed box.
    pub target: Option<NodeId>,
    /// Whether the hover chain changed (any node entered or left).
    pub hover_changed: bool,
    /// Whether the focus target changed (only a press can move focus).
    pub focus_changed: bool,
    /// The command dispatched by this event, if it completed a click on a node
    /// with an `on-click` binding (a release on the same node a prior press
    /// landed on). `None` for every non-click event, and for a click on a node
    /// with no click binding in its ancestor chain.
    pub command: Option<InternedSymbol>,
}

impl PointerDispatch {
    /// Constructs a dispatch outcome.
    #[must_use]
    pub const fn new(
        target: Option<NodeId>,
        hover_changed: bool,
        focus_changed: bool,
        command: Option<InternedSymbol>,
    ) -> Self {
        Self {
            target,
            hover_changed,
            focus_changed,
            command,
        }
    }

    /// The topmost node under the pointer, if any.
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        self.target
    }

    /// Whether the hover chain changed.
    #[must_use]
    pub const fn hover_changed(&self) -> bool {
        self.hover_changed
    }

    /// Whether the focus target changed.
    #[must_use]
    pub const fn focus_changed(&self) -> bool {
        self.focus_changed
    }

    /// The command a completed click dispatched, if any.
    #[must_use]
    pub const fn command(&self) -> Option<InternedSymbol> {
        self.command
    }

    /// Whether this event changed anything (hover or focus) — i.e. whether it
    /// scheduled a frame.
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.hover_changed || self.focus_changed
    }
}
