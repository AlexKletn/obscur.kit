#![doc = include_str!("selector_target.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::NodeId;

#[doc = include_str!("selector_target.doc/SelectorTarget.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorTarget {
    #[doc = include_str!("selector_target.doc/Node.md")]
    Node(NodeId),
    #[doc = include_str!("selector_target.doc/TestId.md")]
    TestId(Box<str>),
    #[doc = include_str!("selector_target.doc/AutomationId.md")]
    AutomationId(Box<str>),
    #[doc = include_str!("selector_target.doc/Coordinate.md")]
    Coordinate(Point),
}

impl SelectorTarget {
    #[doc = include_str!("selector_target.doc/node.md")]
    #[must_use]
    pub const fn node(id: NodeId) -> Self {
        Self::Node(id)
    }

    #[doc = include_str!("selector_target.doc/test_id.md")]
    #[must_use]
    pub fn test_id(id: impl Into<Box<str>>) -> Self {
        Self::TestId(id.into())
    }

    #[doc = include_str!("selector_target.doc/automation_id.md")]
    #[must_use]
    pub fn automation_id(id: impl Into<Box<str>>) -> Self {
        Self::AutomationId(id.into())
    }

    #[doc = include_str!("selector_target.doc/is_automation_id.md")]
    #[must_use]
    pub const fn is_automation_id(&self) -> bool {
        matches!(self, Self::AutomationId(_))
    }

    #[doc = include_str!("selector_target.doc/automation_id_str.md")]
    #[must_use]
    pub fn automation_id_str(&self) -> Option<&str> {
        match self {
            Self::AutomationId(id) => Some(id),
            _ => None,
        }
    }

    #[doc = include_str!("selector_target.doc/coordinate.md")]
    #[must_use]
    pub const fn coordinate(point: Point) -> Self {
        Self::Coordinate(point)
    }

    #[doc = include_str!("selector_target.doc/is_node.md")]
    #[must_use]
    pub const fn is_node(&self) -> bool {
        matches!(self, Self::Node(_))
    }

    #[doc = include_str!("selector_target.doc/is_test_id.md")]
    #[must_use]
    pub const fn is_test_id(&self) -> bool {
        matches!(self, Self::TestId(_))
    }

    #[doc = include_str!("selector_target.doc/is_coordinate.md")]
    #[must_use]
    pub const fn is_coordinate(&self) -> bool {
        matches!(self, Self::Coordinate(_))
    }

    #[doc = include_str!("selector_target.doc/node_id.md")]
    #[must_use]
    pub const fn node_id(&self) -> Option<NodeId> {
        match self {
            Self::Node(id) => Some(*id),
            _ => None,
        }
    }

    #[doc = include_str!("selector_target.doc/coordinate_point.md")]
    #[must_use]
    pub const fn coordinate_point(&self) -> Option<Point> {
        match self {
            Self::Coordinate(point) => Some(*point),
            _ => None,
        }
    }

    #[doc = include_str!("selector_target.doc/test_id_str.md")]
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
