#![doc = include_str!("dispatch_outcome.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};

#[doc = include_str!("dispatch_outcome.doc/DispatchOutcome.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DispatchOutcome {
    #[doc = include_str!("dispatch_outcome.doc/target.md")]
    pub target: Option<NodeId>,
    #[doc = include_str!("dispatch_outcome.doc/propagation_path.md")]
    pub propagation_path: Vec<NodeId>,
    #[doc = include_str!("dispatch_outcome.doc/command.md")]
    pub command: Option<InternedSymbol>,
    #[doc = include_str!("dispatch_outcome.doc/hover_changed.md")]
    pub hover_changed: bool,
    #[doc = include_str!("dispatch_outcome.doc/focus_changed.md")]
    pub focus_changed: bool,
}

impl DispatchOutcome {
    #[doc = include_str!("dispatch_outcome.doc/unresolved.md")]
    #[must_use]
    pub fn unresolved() -> Self {
        Self::default()
    }

    #[doc = include_str!("dispatch_outcome.doc/resolved.md")]
    #[must_use]
    pub const fn resolved(&self) -> bool {
        self.target.is_some()
    }

    #[doc = include_str!("dispatch_outcome.doc/propagates_through.md")]
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
