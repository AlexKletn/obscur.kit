use crate::headless::input::SelectorTarget;
use crate::identity::{InternedSymbol, NodeId};
use crate::interaction::event::ListenerPhase;

use super::SharedHandler;

/// One positional registration: what to run, for which command, at whichever node
/// `target` currently resolves to.
pub(super) struct CommandBinding {
    /// How the node is named — a `test-id`, an `automation-id`, or a direct id
    /// (spec §7.5).
    pub(super) target: SelectorTarget,
    /// The command symbol this registration answers.
    pub(super) command: InternedSymbol,
    /// `None` for a §2.6 handler (link 1/3 of the routing chain), `Some(phase)` for
    /// a §2.3 listener in that phase.
    pub(super) phase: Option<ListenerPhase>,
    /// The application's handler, shared so a re-bind does not consume it.
    pub(super) handler: SharedHandler,
    /// The node this is currently registered at, if it has resolved yet.
    pub(super) bound: Option<NodeId>,
}
