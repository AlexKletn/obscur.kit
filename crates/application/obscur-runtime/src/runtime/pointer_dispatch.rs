//! The outcome of dispatching one pointer event: [`PointerDispatch`].

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::EventTrust;

#[doc = include_str!("../../docs/api/runtime/pointer_dispatch.doc/PointerDispatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointerDispatch {
    /// The topmost node under the pointer after hit-testing, or `None` when the
    /// pointer is over no indexed box.
    pub target: Option<NodeId>,
    /// Whether the hover chain changed (any node entered or left).
    pub hover_changed: bool,
    /// Whether the focus target changed (only a press can move focus).
    pub focus_changed: bool,
    /// Whether a raw pointer or context-menu handler called `prevent_default`.
    pub default_prevented: bool,
    /// The command dispatched by this event, if it completed a click on a node
    /// with an `on-click` binding (a release on the same node a prior press
    /// landed on). `None` for every non-click event, and for a click on a node
    /// with no click binding in its ancestor chain.
    pub command: Option<InternedSymbol>,
    #[doc = include_str!("../../docs/api/runtime/pointer_dispatch.doc/current_target.md")]
    pub current_target: Option<NodeId>,
    #[doc = include_str!("../../docs/api/runtime/pointer_dispatch.doc/trust.md")]
    pub trust: EventTrust,
}

impl PointerDispatch {
    /// Constructs a dispatch outcome.
    #[must_use]
    pub const fn new(
        target: Option<NodeId>,
        hover_changed: bool,
        focus_changed: bool,
        command: Option<InternedSymbol>,
        current_target: Option<NodeId>,
        trust: EventTrust,
    ) -> Self {
        Self {
            target,
            hover_changed,
            focus_changed,
            default_prevented: false,
            command,
            current_target,
            trust,
        }
    }

    pub(crate) const fn with_default_prevented(mut self, prevented: bool) -> Self {
        self.default_prevented = prevented;
        self
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

    /// §2.3's `current_target`: the node the fired handler is attached to.
    #[must_use]
    pub const fn current_target(&self) -> Option<NodeId> {
        self.current_target
    }

    /// §2.3's trusted/untrusted origin for the event that produced this dispatch.
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    /// Whether the event that produced this dispatch was a real user gesture.
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
    }

    /// Whether this event changed anything (hover or focus) — i.e. whether it
    /// scheduled a frame.
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.hover_changed || self.focus_changed
    }
}
