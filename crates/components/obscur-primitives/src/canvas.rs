#![doc = include_str!("canvas.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewNode, ViewProperty, ViewValue, canvas_painter_prop};

#[doc = include_str!("canvas.doc/Canvas.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    width: f32,
    height: f32,
    painter: Option<InternedSymbol>,
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
        Self {
            width,
            height,
            painter: None,
        }
    }

    #[doc = include_str!("canvas.doc/named.md")]
    #[must_use]
    pub const fn named(mut self, painter: InternedSymbol) -> Self {
        self.painter = Some(painter);
        self
    }

    #[doc = include_str!("canvas.doc/props.md")]
    #[must_use]
    pub fn props(&self) -> Vec<ViewProperty> {
        let mut props = vec![
            ViewProperty::new(Self::PROP_WIDTH, ViewValue::Float(f64::from(self.width))),
            ViewProperty::new(Self::PROP_HEIGHT, ViewValue::Float(f64::from(self.height))),
        ];
        if let Some(painter) = self.painter {
            props.push(canvas_painter_prop(painter));
        }
        props
    }

    #[doc = include_str!("canvas.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        let props = self.props();
        let mut node = ViewNode::element(Self::COMPONENT_ID);
        for prop in props {
            node = node.with_prop(prop);
        }
        node
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
    use obscur_view::view::canvas_painter;

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
    fn a_named_canvas_carries_its_painter_and_an_unnamed_one_carries_none() {
        let named = Canvas::new(1.0, 1.0).named(InternedSymbol::from_raw(0xABCD));
        assert_eq!(
            canvas_painter(&named.props()),
            Some(InternedSymbol::from_raw(0xABCD))
        );
        assert_eq!(
            canvas_painter(&Canvas::new(1.0, 1.0).props()),
            None,
            "a canvas that names no painter is an empty box, as it always was"
        );
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Canvas::new(1.0, 1.0).view();
        assert_eq!(view.root().len(), 1);
    }
}
