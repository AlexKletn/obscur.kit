//! How a synthetic input (or a headless query) locates its node (spec §7.4, §7.5).

use obscur_geometry::Point;
use obscur_identity::NodeId;

/// Where a synthetic input (or a headless query) is aimed.
///
/// Testing facilities target a node either directly, by a stable selector, or by
/// a viewport coordinate (spec §7.4). Stable selectors are independent of CSS and
/// do not affect layout or visuals (spec §7.5).
///
/// Resolution semantics ([`SelectorResolver`](crate::input::SelectorResolver)):
/// - [`SelectorTarget::Node`] names a node directly; it resolves only while that
///   node is still live.
/// - [`SelectorTarget::TestId`] resolves by matching the node's `test-id`
///   property. It is not a style attribute, so it affects no matching, layout or
///   visual output (spec §7.5).
/// - [`SelectorTarget::Coordinate`] is *not* resolved here: it is aimed straight
///   at [`UiRuntime::handle_pointer`](obscur_runtime::UiRuntime::handle_pointer),
///   which hit-tests it exactly as it would a real pointer. Re-implementing that
///   would give a second, divergent answer.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorTarget {
    /// A direct, already-resolved stable node id.
    Node(NodeId),
    /// A stable `test-id` string, independent of CSS/layout (spec §7.5). The
    /// string is data matched during dispatch, never used for control-flow dispatch.
    TestId(Box<str>),
    /// A point in the viewport; the node is found by hit-testing during dispatch.
    Coordinate(Point),
}

impl SelectorTarget {
    /// Targets an already-resolved stable node id directly.
    #[must_use]
    pub const fn node(id: NodeId) -> Self {
        Self::Node(id)
    }

    /// Targets a node by its stable `test-id` string (resolved during dispatch).
    #[must_use]
    pub fn test_id(id: impl Into<Box<str>>) -> Self {
        Self::TestId(id.into())
    }

    /// Targets a viewport point, resolved to a node by hit-testing during dispatch.
    #[must_use]
    pub const fn coordinate(point: Point) -> Self {
        Self::Coordinate(point)
    }

    /// Returns `true` if this is a direct [`SelectorTarget::Node`] target.
    #[must_use]
    pub const fn is_node(&self) -> bool {
        matches!(self, Self::Node(_))
    }

    /// Returns `true` if this is a [`SelectorTarget::TestId`] target.
    #[must_use]
    pub const fn is_test_id(&self) -> bool {
        matches!(self, Self::TestId(_))
    }

    /// Returns `true` if this is a [`SelectorTarget::Coordinate`] target.
    #[must_use]
    pub const fn is_coordinate(&self) -> bool {
        matches!(self, Self::Coordinate(_))
    }

    /// Returns the node id for a [`SelectorTarget::Node`] target, else `None`.
    #[must_use]
    pub const fn node_id(&self) -> Option<NodeId> {
        match self {
            Self::Node(id) => Some(*id),
            _ => None,
        }
    }

    /// Returns the point for a [`SelectorTarget::Coordinate`] target, else `None`.
    #[must_use]
    pub const fn coordinate_point(&self) -> Option<Point> {
        match self {
            Self::Coordinate(point) => Some(*point),
            _ => None,
        }
    }

    /// Returns the `test-id` string for a [`SelectorTarget::TestId`] target, else `None`.
    #[must_use]
    pub fn test_id_str(&self) -> Option<&str> {
        match self {
            Self::TestId(id) => Some(id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectorTarget;
    use obscur_geometry::Point;
    use obscur_identity::NodeId;

    #[test]
    fn node_constructor_and_accessors() {
        let id = NodeId::from_raw(7);
        let target = SelectorTarget::node(id);

        assert!(target.is_node());
        assert!(!target.is_test_id());
        assert!(!target.is_coordinate());
        assert_eq!(target.node_id(), Some(id));
        assert_eq!(target.coordinate_point(), None);
        assert_eq!(target.test_id_str(), None);
    }

    #[test]
    fn test_id_constructor_and_accessors() {
        let target = SelectorTarget::test_id("submit-button");

        assert!(target.is_test_id());
        assert!(!target.is_node());
        assert!(!target.is_coordinate());
        assert_eq!(target.test_id_str(), Some("submit-button"));
        assert_eq!(target.node_id(), None);
        assert_eq!(target.coordinate_point(), None);
    }

    #[test]
    fn coordinate_constructor_and_accessors() {
        let point = Point::new(12.0, 34.0);
        let target = SelectorTarget::coordinate(point);

        assert!(target.is_coordinate());
        assert!(!target.is_node());
        assert!(!target.is_test_id());
        assert_eq!(target.coordinate_point(), Some(point));
        assert_eq!(target.node_id(), None);
        assert_eq!(target.test_id_str(), None);
    }

    #[test]
    fn equality_is_by_value() {
        assert_eq!(
            SelectorTarget::node(NodeId::from_raw(1)),
            SelectorTarget::node(NodeId::from_raw(1)),
        );
        assert_ne!(SelectorTarget::test_id("a"), SelectorTarget::test_id("b"),);
    }
}
