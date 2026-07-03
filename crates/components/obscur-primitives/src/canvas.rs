//! `Canvas`: a custom-rendering surface primitive (spec §2.6, §5.3).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewNode, ViewProperty, ViewValue};

/// A custom-rendering surface that participates in layout and hit-testing.
///
/// `Canvas` is one of the minimal built-in primitives (spec §2.6). It reserves
/// a region the application paints itself (spec §5.3) and carries an intrinsic
/// size hint so the layout engine can size it before the custom paint runs. It
/// is a leaf element primitive with no children: it lowers to a
/// [`ViewNode::Element`] carrying its width and height as properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    width: f32,
    height: f32,
}

impl Canvas {
    /// Stable component id for the canvas primitive (see
    /// [`Container::COMPONENT_ID`] for the shared id scheme).
    ///
    /// [`Container::COMPONENT_ID`]: crate::Container::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0005);

    /// Property name for the intrinsic width hint (see [`Image::PROP_SRC`] for
    /// the shared prop-symbol scheme).
    ///
    /// [`Image::PROP_SRC`]: crate::Image::PROP_SRC
    pub const PROP_WIDTH: InternedSymbol = InternedSymbol::from_raw(0x00C1_0501);
    /// Property name for the intrinsic height hint.
    pub const PROP_HEIGHT: InternedSymbol = InternedSymbol::from_raw(0x00C1_0502);

    /// Creates a canvas with the given intrinsic size hint in logical pixels.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Lowers this canvas into its [`ViewNode::Element`], emitting its width and
    /// height as float properties.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_WIDTH,
                ViewValue::Float(f64::from(self.width)),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_HEIGHT,
                ViewValue::Float(f64::from(self.height)),
            ))
    }

    /// Lowers this canvas into a single-node [`View`].
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_size_hints() {
        let node = Canvas::new(320.0, 240.0).into_view_node();
        if let ViewNode::Element {
            component, props, ..
        } = node
        {
            assert_eq!(component, Canvas::COMPONENT_ID);
            assert_eq!(props[0].name(), Canvas::PROP_WIDTH);
            assert_eq!(props[0].value(), &ViewValue::Float(320.0));
            assert_eq!(props[1].name(), Canvas::PROP_HEIGHT);
            assert_eq!(props[1].value(), &ViewValue::Float(240.0));
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Canvas::new(1.0, 1.0).view();
        assert_eq!(view.root().len(), 1);
    }
}
