//! `Portal`: a remote-render primitive (spec §2.6).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A node that renders its children at a named target elsewhere in the tree.
///
/// `Portal` is one of the minimal built-in primitives (spec §2.6). Its children
/// stay logically parented here (for data flow and events) but are visually
/// projected to a named portal target — the mechanism behind overlays, dialogs,
/// and tooltips that must escape their ancestor's clip or z-order. It is a leaf
/// element primitive: it lowers to a [`ViewNode::Element`] carrying the target
/// name as a property and its children as the element's fragment.
///
/// Unlike the childless text/image/foreign-surface primitives, `Portal` cannot
/// derive `Eq`: it holds child [`ViewNode`]s, which are only `PartialEq`
/// because their property values carry an `f64`.
#[derive(Debug, Clone, PartialEq)]
pub struct Portal {
    target: Box<str>,
    children: Vec<ViewNode>,
}

impl Portal {
    /// Stable component id for the portal primitive (see
    /// [`Container::COMPONENT_ID`] for the shared id scheme).
    ///
    /// [`Container::COMPONENT_ID`]: crate::Container::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0006);

    /// Property name for the portal target name (see [`Image::PROP_SRC`] for
    /// the shared prop-symbol scheme).
    ///
    /// [`Image::PROP_SRC`]: crate::Image::PROP_SRC
    pub const PROP_TARGET: InternedSymbol = InternedSymbol::from_raw(0x00C1_0601);

    /// Creates a portal projecting to the named `target` with no children.
    #[must_use]
    pub fn new(target: impl Into<Box<str>>) -> Self {
        Self {
            target: target.into(),
            children: Vec::new(),
        }
    }

    /// Appends a child node, returning the updated portal.
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    /// Lowers this portal into its [`ViewNode::Element`], emitting the target
    /// property and its children as the fragment.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_TARGET,
                ViewValue::Text(self.target),
            ))
            .with_children(ViewFragment::from_nodes(self.children))
    }

    /// Lowers this portal into a single-node [`View`].
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_target_and_children() {
        let node = Portal::new("overlay-root")
            .child(ViewNode::text("dialog"))
            .into_view_node();
        if let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        {
            assert_eq!(component, Portal::COMPONENT_ID);
            assert_eq!(props[0].name(), Portal::PROP_TARGET);
            assert_eq!(props[0].value(), &ViewValue::Text("overlay-root".into()));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Portal::new("t").view();
        assert_eq!(view.root().len(), 1);
    }
}
