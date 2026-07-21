//! The outcome of dispatching one pointer event: [`PointerDispatch`].

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::EventTrust;

/// The result of routing a pointer event through
/// [`UiRuntime::handle_pointer`](super::UiRuntime::handle_pointer).
///
/// It reports the node the pointer resolved to (the topmost hit, if any) and
/// whether this event actually moved the hover chain or the focus target —
/// the two signals a host uses to decide whether a restyle/repaint is warranted.
/// When both `hover_changed` and `focus_changed` are `false`, the runtime's
/// interaction state is byte-for-byte unchanged and no frame was requested.
///
/// It also carries the two §2.3 facts a host cannot reconstruct for itself:
/// [`current_target`](Self::current_target) — the node the fired handler is
/// *attached to*, which is not in general the node the pointer *hit* — and
/// [`trust`](Self::trust), whether the event was a real user gesture.
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
    /// §2.3's `current_target`: the node whose binding resolved
    /// [`command`](Self::command) — the element the handler is *attached to*, as
    /// against [`target`](Self::target), the element the event *hit*.
    ///
    /// Equal to `target` when the hit node carries the binding itself, and an
    /// **ancestor** of `target` when the event bubbled to find one. `None`
    /// exactly when `command` is `None`: no binding resolved, so no handler ran,
    /// so nothing is a current target.
    pub current_target: Option<NodeId>,
    /// §2.3's trusted/untrusted origin, carried through from the
    /// [`PointerEvent`](obscur_interaction::pointer::PointerEvent) that produced
    /// this dispatch.
    ///
    /// [`Trusted`](EventTrust::Trusted) for input a driver vouched for: a platform
    /// adapter converting a real OS event, and the first-party **headless** test
    /// harness (§7.4) standing in for a real user. [`Untrusted`](EventTrust::Untrusted)
    /// for input no one vouches for: the public programmatic
    /// [`handle_pointer`](super::UiRuntime::handle_pointer) API called with a bare
    /// event, and an external DevTools `Inject` (§7.6) arriving over the socket.
    ///
    /// The runtime **gates the default action** on this (§2.3 step 4): a click's
    /// activation — its bound command and handler — runs only for a trusted
    /// gesture, so an untrusted event still dispatches and is observed (it
    /// hit-tests, and hover/focus still move) but resolves
    /// [`command`](Self::command) to `None` and fires no handler. This keeps host
    /// control over a consequential action behind a real gesture (§1.8, §9): an
    /// external agent may drive observation, not the gated action.
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
            command,
            current_target,
            trust,
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
