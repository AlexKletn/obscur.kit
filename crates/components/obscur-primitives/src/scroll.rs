//! `Scroll`: a scrollable-viewport primitive (spec §2.6, §4.8).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A scrollable viewport wrapping a set of children.
///
/// `Scroll` is one of the minimal built-in primitives (spec §2.6). It clips its
/// children to a viewport and offers scrolling along the enabled axes
/// (spec §4.8). By default it scrolls vertically only; either axis can be
/// toggled. It is a leaf element primitive: it lowers to a
/// [`ViewNode::Element`] carrying its per-axis flags as properties and its
/// children as the element's fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct Scroll {
    children: Vec<ViewNode>,
    horizontal: bool,
    vertical: bool,
}

impl Default for Scroll {
    /// A vertical-only viewport with no children.
    fn default() -> Self {
        Self {
            children: Vec::new(),
            horizontal: false,
            vertical: true,
        }
    }
}

impl Scroll {
    /// Stable component id for the scroll primitive (see
    /// [`Container::COMPONENT_ID`] for the shared id scheme).
    ///
    /// [`Container::COMPONENT_ID`]: crate::Container::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0004);

    /// Property name for the horizontal-scroll flag (see [`Image::PROP_SRC`]
    /// for the shared prop-symbol scheme).
    ///
    /// [`Image::PROP_SRC`]: crate::Image::PROP_SRC
    pub const PROP_SCROLL_X: InternedSymbol = InternedSymbol::from_raw(0x00C1_0401);
    /// Property name for the vertical-scroll flag.
    pub const PROP_SCROLL_Y: InternedSymbol = InternedSymbol::from_raw(0x00C1_0402);

    /// Creates a default scroll viewport (vertical only, no children).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a child node, returning the updated viewport.
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    /// Sets whether the viewport scrolls horizontally, returning the updated
    /// viewport.
    #[must_use]
    pub const fn horizontal(mut self, enabled: bool) -> Self {
        self.horizontal = enabled;
        self
    }

    /// Sets whether the viewport scrolls vertically, returning the updated
    /// viewport.
    #[must_use]
    pub const fn vertical(mut self, enabled: bool) -> Self {
        self.vertical = enabled;
        self
    }

    /// Lowers this viewport into its [`ViewNode::Element`], emitting both axis
    /// flags as properties and its children as the fragment.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_SCROLL_X,
                ViewValue::Bool(self.horizontal),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_SCROLL_Y,
                ViewValue::Bool(self.vertical),
            ))
            .with_children(ViewFragment::from_nodes(self.children))
    }

    /// Lowers this viewport into a single-node [`View`].
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_vertical_only() {
        let s = Scroll::new();
        assert!(!s.horizontal);
        assert!(s.vertical);
    }

    #[test]
    fn emits_axis_flags_and_children() {
        let node = Scroll::new()
            .horizontal(true)
            .vertical(false)
            .child(ViewNode::text("a"))
            .into_view_node();
        if let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        {
            assert_eq!(component, Scroll::COMPONENT_ID);
            assert_eq!(props[0].name(), Scroll::PROP_SCROLL_X);
            assert_eq!(props[0].value(), &ViewValue::Bool(true));
            assert_eq!(props[1].name(), Scroll::PROP_SCROLL_Y);
            assert_eq!(props[1].value(), &ViewValue::Bool(false));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Scroll::new().view();
        assert_eq!(view.root().len(), 1);
    }
}
