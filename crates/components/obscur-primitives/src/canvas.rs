#![doc = include_str!("canvas.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("canvas.doc/Canvas.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    width: f32,
    height: f32,
}

impl Canvas {
    #[doc = include_str!("canvas.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0005);

    #[doc = include_str!("canvas.doc/PROP_WIDTH.md")]
    pub const PROP_WIDTH: InternedSymbol = InternedSymbol::from_raw(0x00C1_0501);
    #[doc = include_str!("canvas.doc/PROP_HEIGHT.md")]
    pub const PROP_HEIGHT: InternedSymbol = InternedSymbol::from_raw(0x00C1_0502);

    #[doc = include_str!("canvas.doc/new.md")]
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[doc = include_str!("canvas.doc/into_view_node.md")]
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

    #[doc = include_str!("canvas.doc/view.md")]
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
