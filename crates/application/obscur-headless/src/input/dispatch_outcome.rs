//! `DispatchOutcome`: the typed result of dispatching one synthetic input.

use obscur_identity::{InternedSymbol, NodeId};

/// What a dispatched [`SyntheticInput`](crate::input::SyntheticInput) actually
/// did (spec §7.4 "typed dispatch results").
///
/// Every field is observed from the *real* input path — the same
/// [`UiRuntime`](obscur_runtime::UiRuntime) call a windowed host makes — so a
/// headless assertion cannot pass while a real click would fail.
///
/// # What is and is not here
///
/// The spec lists target, propagation path, default action, focus changes, state
/// changes and layout changes. This carries what the runtime genuinely reports
/// today: the resolved [`target`](Self::target), the
/// [`propagation_path`](Self::propagation_path) (target → root, the chain a
/// command binding is looked up along), the dispatched
/// [`command`](Self::command) — the default action of a completed click or a
/// bound key — and whether [`hover`](Self::hover_changed) or
/// [`focus`](Self::focus_changed) moved.
///
/// Explicit state-change and layout-change records are **not** included, because
/// the runtime does not produce them: reporting them would mean inventing a
/// second, unverified source of truth. A test asserts those by inspecting the
/// tree and layout boxes it already has.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DispatchOutcome {
    /// The node the input landed on, or [`None`] when it resolved to nothing
    /// (an unknown `test-id`, a coordinate over no box, a key with no focus).
    pub target: Option<NodeId>,
    /// The chain from [`target`](Self::target) to the root, inclusive — the path
    /// an event's default action is resolved along. Empty when there is no target.
    pub propagation_path: Vec<NodeId>,
    /// The command the input dispatched, if it triggered one: a completed click
    /// on a node with a binding in its propagation path, or a bound shortcut.
    /// This is the event's *default action*.
    pub command: Option<InternedSymbol>,
    /// Whether the hover chain changed.
    pub hover_changed: bool,
    /// Whether the focus target changed.
    pub focus_changed: bool,
}

impl DispatchOutcome {
    /// An outcome for an input that resolved to nothing and did nothing.
    #[must_use]
    pub fn unresolved() -> Self {
        Self::default()
    }

    /// Whether the input found a node to act on.
    #[must_use]
    pub const fn resolved(&self) -> bool {
        self.target.is_some()
    }

    /// Whether `node` is on the propagation path.
    #[must_use]
    pub fn propagates_through(&self, node: NodeId) -> bool {
        self.propagation_path.contains(&node)
    }
}

#[cfg(test)]
mod tests {
    use super::DispatchOutcome;
    use obscur_identity::NodeId;

    #[test]
    fn an_unresolved_outcome_reports_nothing_happened() {
        let outcome = DispatchOutcome::unresolved();
        assert!(!outcome.resolved());
        assert!(outcome.propagation_path.is_empty());
        assert_eq!(outcome.command, None);
        assert!(!outcome.hover_changed && !outcome.focus_changed);
    }

    #[test]
    fn propagation_membership_is_by_node() {
        let outcome = DispatchOutcome {
            target: Some(NodeId::from_raw(2)),
            propagation_path: vec![NodeId::from_raw(2), NodeId::from_raw(1)],
            ..DispatchOutcome::unresolved()
        };
        assert!(outcome.propagates_through(NodeId::from_raw(1)));
        assert!(!outcome.propagates_through(NodeId::from_raw(9)));
    }
}
