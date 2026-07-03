//! The [`TabOrder`] computing the tab navigation sequence (spec §7.1 focus).

use obscur_identity::NodeId;

use super::FocusNode;

/// The resolved tab navigation sequence for a set of focus nodes (spec §7.1).
///
/// [`compute`](Self::compute) derives the sequence from a slice of
/// [`FocusNode`]s. Only [`is_tab_focusable`](FocusNode::is_tab_focusable) nodes
/// participate (excluding negative `tab_index` and disabled nodes). The ordering
/// mirrors HTML `tabindex`:
///
/// 1. nodes with a positive `tab_index`, in ascending `tab_index` order; ties
///    keep their original (input) order;
/// 2. followed by nodes with `tab_index == 0`, in their original order.
///
/// The sort is stable, so the sequence is fully deterministic given the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabOrder {
    ordered: Vec<NodeId>,
}

impl TabOrder {
    /// Computes the tab sequence over `nodes`, keeping only tab-focusable nodes
    /// and ordering them by tab-index priority (positive ascending, then `0`),
    /// preserving input order within each group.
    #[must_use]
    pub fn compute(nodes: &[FocusNode]) -> Self {
        let mut eligible: Vec<&FocusNode> =
            nodes.iter().filter(|n| n.is_tab_focusable()).collect();
        // Stable sort: positive tab_index (group 0) ascending, then tab_index==0
        // (group 1). Equal keys retain their original relative order.
        eligible.sort_by_key(|n| {
            if n.tab_index() > 0 {
                (0_i32, n.tab_index())
            } else {
                (1_i32, 0_i32)
            }
        });
        Self {
            ordered: eligible.iter().map(|n| n.node()).collect(),
        }
    }

    /// Returns the resolved tab sequence.
    #[must_use]
    pub fn nodes(&self) -> &[NodeId] {
        &self.ordered
    }

    /// Returns the first node in the sequence, if any.
    #[must_use]
    pub fn first(&self) -> Option<NodeId> {
        self.ordered.first().copied()
    }

    /// Returns the last node in the sequence, if any.
    #[must_use]
    pub fn last(&self) -> Option<NodeId> {
        self.ordered.last().copied()
    }

    /// Returns the node following `current` in tab order.
    ///
    /// With `current` `None` the first node is returned. When `current` is the
    /// last node (or is not present in the sequence at its end), the first node
    /// is returned if `wrap` is set, otherwise `None`.
    #[must_use]
    pub fn next_after(&self, current: Option<NodeId>, wrap: bool) -> Option<NodeId> {
        let Some(current) = current else {
            return self.first();
        };
        match self.ordered.iter().position(|n| *n == current) {
            Some(index) if index + 1 < self.ordered.len() => Some(self.ordered[index + 1]),
            Some(_) => {
                if wrap {
                    self.first()
                } else {
                    None
                }
            }
            // `current` is not part of the sequence: start from the beginning.
            None => self.first(),
        }
    }

    /// Returns the node preceding `current` in tab order.
    ///
    /// With `current` `None` the last node is returned. When `current` is the
    /// first node (or is not present in the sequence), the last node is returned
    /// if `wrap` is set, otherwise `None`.
    #[must_use]
    pub fn previous_before(&self, current: Option<NodeId>, wrap: bool) -> Option<NodeId> {
        let Some(current) = current else {
            return self.last();
        };
        match self.ordered.iter().position(|n| *n == current) {
            Some(index) if index > 0 => Some(self.ordered[index - 1]),
            Some(_) => {
                if wrap {
                    self.last()
                } else {
                    None
                }
            }
            // `current` is not part of the sequence: start from the end.
            None => self.last(),
        }
    }

    /// Returns whether `node` is part of the tab sequence.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.ordered.contains(&node)
    }

    /// Returns the number of nodes in the sequence.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    /// Returns whether the sequence is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn focusable(raw: u64, tab_index: i32) -> FocusNode {
        FocusNode::new(node(raw)).focusable(true).with_tab_index(tab_index)
    }

    #[test]
    fn positive_tab_index_sorts_before_zero_ascending() {
        // Input order deliberately scrambled to exercise the sort.
        let nodes = [
            focusable(1, 0),
            focusable(2, 2),
            focusable(3, 0),
            focusable(4, 1),
        ];
        let order = TabOrder::compute(&nodes);
        // Positives ascending (4@1, 2@2), then zeros in input order (1, 3).
        assert_eq!(order.nodes(), &[node(4), node(2), node(1), node(3)]);
    }

    #[test]
    fn equal_tab_index_is_stable() {
        let nodes = [focusable(1, 3), focusable(2, 3), focusable(3, 3)];
        let order = TabOrder::compute(&nodes);
        assert_eq!(order.nodes(), &[node(1), node(2), node(3)]);
    }

    #[test]
    fn negative_and_disabled_are_excluded() {
        let nodes = [
            focusable(1, -1),
            focusable(2, 0),
            FocusNode::new(node(3)).focusable(true).disabled(true),
            FocusNode::new(node(4)), // not focusable
        ];
        let order = TabOrder::compute(&nodes);
        assert_eq!(order.nodes(), &[node(2)]);
    }

    #[test]
    fn next_after_without_wrap() {
        let nodes = [focusable(1, 0), focusable(2, 0), focusable(3, 0)];
        let order = TabOrder::compute(&nodes);
        assert_eq!(order.next_after(None, false), Some(node(1)));
        assert_eq!(order.next_after(Some(node(1)), false), Some(node(2)));
        assert_eq!(order.next_after(Some(node(3)), false), None);
    }

    #[test]
    fn next_after_with_wrap() {
        let nodes = [focusable(1, 0), focusable(2, 0)];
        let order = TabOrder::compute(&nodes);
        assert_eq!(order.next_after(Some(node(2)), true), Some(node(1)));
    }

    #[test]
    fn previous_before_with_and_without_wrap() {
        let nodes = [focusable(1, 0), focusable(2, 0), focusable(3, 0)];
        let order = TabOrder::compute(&nodes);
        assert_eq!(order.previous_before(None, false), Some(node(3)));
        assert_eq!(order.previous_before(Some(node(2)), false), Some(node(1)));
        assert_eq!(order.previous_before(Some(node(1)), false), None);
        assert_eq!(order.previous_before(Some(node(1)), true), Some(node(3)));
    }

    #[test]
    fn helpers_report_bounds() {
        let empty = TabOrder::compute(&[]);
        assert!(empty.is_empty());
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);

        let nodes = [focusable(1, 0), focusable(2, 0)];
        let order = TabOrder::compute(&nodes);
        assert_eq!(order.len(), 2);
        assert!(order.contains(node(1)));
        assert!(!order.contains(node(9)));
        assert_eq!(order.first(), Some(node(1)));
        assert_eq!(order.last(), Some(node(2)));
    }
}
