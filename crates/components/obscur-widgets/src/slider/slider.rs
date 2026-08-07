#![doc = include_str!("slider.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Canvas;
use obscur_view::view::{
    LOCAL_NAME, SLIDER_HIGH, SLIDER_LOW, SLIDER_MAX, SLIDER_MIN, SLIDER_RANGED, SLIDER_STEP,
    SliderScale, View, ViewFragment, ViewNode, ViewProperty, ViewValue, canvas_painter_prop,
};

#[doc = include_str!("slider.doc/Slider.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slider {
    #[doc = include_str!("slider.doc/scale.md")]
    scale: SliderScale,
    #[doc = include_str!("slider.doc/low.md")]
    low: f32,
    #[doc = include_str!("slider.doc/high.md")]
    high: f32,
    #[doc = include_str!("slider.doc/ranged.md")]
    ranged: bool,
}

impl Slider {
    #[doc = include_str!("slider.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0008);

    #[doc = include_str!("slider.doc/PAINTER.md")]
    pub const PAINTER: InternedSymbol = InternedSymbol::from_raw(0x00D1_0801);

    #[doc = include_str!("slider.doc/new.md")]
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self {
            scale: SliderScale::default(),
            low: value,
            high: value,
            ranged: false,
        }
    }

    #[doc = include_str!("slider.doc/ranging.md")]
    #[must_use]
    pub fn ranging(low: f32, high: f32) -> Self {
        let scale = SliderScale::default();
        Self {
            scale,
            low,
            high,
            ranged: true,
        }
    }

    #[doc = include_str!("slider.doc/over.md")]
    #[must_use]
    pub fn over(mut self, min: f32, max: f32) -> Self {
        self.scale = SliderScale {
            min,
            max,
            step: self.scale.step,
        };
        self
    }

    #[doc = include_str!("slider.doc/stepped_by.md")]
    #[must_use]
    pub fn stepped_by(mut self, step: f32) -> Self {
        self.scale = self.scale.stepped_by(step);
        self
    }

    #[doc = include_str!("slider.doc/value.md")]
    #[must_use]
    pub fn value(&self) -> f32 {
        self.range().0
    }

    #[doc = include_str!("slider.doc/range.md")]
    #[must_use]
    pub fn range(&self) -> (f32, f32) {
        // Snapped and sorted HERE — on the way out — and not as the builder
        // runs. A builder that quantized eagerly would destroy the author's
        // value before the scale it belongs to had been stated:
        // `Slider::new(6.4).over(0.0, 10.0)` would have clamped 6.4 against the
        // default 0..1 and answered 1. Order of construction is not something a
        // caller should have to know.
        //
        // Sorted rather than clamped: clamping the high up to the low would
        // collapse a reversed range to a point and throw away one of the two
        // numbers the author wrote, when reversed ends almost always mean the
        // pair.
        let (low, high) = (
            self.scale.quantize(self.low),
            self.scale.quantize(self.high),
        );
        (low.min(high), low.max(high))
    }

    #[doc = include_str!("slider.doc/scale_2.md")]
    #[must_use]
    pub const fn scale(&self) -> SliderScale {
        self.scale
    }

    #[doc = include_str!("slider.doc/props.md")]
    #[must_use]
    pub fn props(&self) -> Vec<ViewProperty> {
        let mut props = vec![
            ViewProperty::new(LOCAL_NAME, ViewValue::Text("slider".into())),
            ViewProperty::new(SLIDER_MIN, ViewValue::Float(f64::from(self.scale.min))),
            ViewProperty::new(SLIDER_MAX, ViewValue::Float(f64::from(self.scale.max))),
            ViewProperty::new(SLIDER_LOW, ViewValue::Float(f64::from(self.range().0))),
            ViewProperty::new(SLIDER_HIGH, ViewValue::Float(f64::from(self.range().1))),
            ViewProperty::new(SLIDER_RANGED, ViewValue::Bool(self.ranged)),
        ];
        if let Some(step) = self.scale.step {
            props.push(ViewProperty::new(
                SLIDER_STEP,
                ViewValue::Float(f64::from(step)),
            ));
        }
        props
    }

    #[doc = include_str!("slider.doc/contents.md")]
    #[must_use]
    pub fn contents(&self) -> ViewFragment {
        // Everything the painter reads rides on the canvas, so the host node and
        // the canvas carry the same facts. The host's copy is what the cascade,
        // the accessibility projection and the runtime's drag will read; the
        // canvas's copy is what draws.
        let mut node = Canvas::new(0.0, 0.0)
            .into_view_node()
            .with_prop(canvas_painter_prop(Self::PAINTER));
        for prop in self.props() {
            if prop.name() != LOCAL_NAME {
                node = node.with_prop(prop);
            }
        }
        ViewFragment::from_nodes(vec![node])
    }

    #[doc = include_str!("slider.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let mut node = ViewNode::element(Self::COMPONENT_ID);
        for prop in self.props() {
            node = node.with_prop(prop);
        }
        View::single(node.with_children(self.contents()))
    }
}

#[cfg(test)]
mod tests {
    use super::Slider;
    use obscur_view::view::{LOCAL_NAME, SLIDER_LOW, SliderScale, ViewNode, ViewValue};

    #[test]
    fn a_value_is_snapped_to_the_scale_it_is_given() {
        // And re-snapped when the scale changes under it: a slider whose value
        // was off its own grid would hand a drag a starting point the control
        // cannot return to.
        // `new` takes a VALUE, not a fraction, and widening the scale does not
        // reinterpret it: 0.7 stays 0.7 on 0..10, then snaps to the nearest
        // multiple of 2, which is 0.
        assert_eq!(
            Slider::new(0.7).over(0.0, 10.0).stepped_by(2.0).value(),
            0.0
        );
        assert_eq!(
            Slider::new(6.4).over(0.0, 10.0).stepped_by(2.0).value(),
            6.0,
            "the scale is stated after the value and must not have destroyed it"
        );
        assert_eq!(
            Slider::new(6.4).stepped_by(2.0).over(0.0, 10.0).value(),
            6.0,
            "and the order the builder is called in does not matter"
        );
        assert_eq!(
            Slider::new(0.44).over(0.0, 10.0).value(),
            0.44,
            "a free scale does not snap"
        );
    }

    #[test]
    fn a_range_orders_its_ends_once_and_here() {
        // Reversed ends almost always mean the pair and not the point, so they
        // are SORTED rather than clamped — clamping would throw one of the two
        // numbers the author wrote away.
        let backwards = Slider::ranging(0.8, 0.2);
        assert_eq!(backwards.range(), (0.2, 0.8));
        assert_eq!(Slider::ranging(0.2, 0.8).range(), (0.2, 0.8));
    }

    #[test]
    fn a_single_slider_is_a_range_of_one_point() {
        // Which is what lets the painter read both ends uniformly instead of
        // branching on `ranged` for its geometry.
        let single = Slider::new(0.3);
        assert_eq!(single.range(), (0.3, 0.3));
        assert_eq!(single.value(), 0.3);
    }

    #[test]
    fn the_host_and_its_canvas_carry_the_same_facts() {
        let slider = Slider::ranging(0.2, 0.6).over(0.0, 1.0);
        let view = slider.view();
        let ViewNode::Element {
            props, children, ..
        } = &view.root().nodes()[0]
        else {
            panic!("a slider is an element");
        };
        assert!(
            props
                .iter()
                .any(|p| p.name() == LOCAL_NAME && *p.value() == ViewValue::Text("slider".into()))
        );

        let ViewNode::Element { props: bar, .. } = &children.nodes()[0] else {
            panic!("its bar is an element");
        };
        let low_of = |set: &[obscur_view::view::ViewProperty]| {
            set.iter()
                .find(|p| p.name() == SLIDER_LOW)
                .map(|p| p.value().clone())
        };
        assert_eq!(low_of(props), low_of(bar), "one truth, written twice");
        // The bar carries its OWN name — `Canvas::into_view_node` names every
        // canvas `canvas`, which is what lets a stylesheet reach the box a painter
        // draws into — and it must never carry the host's. `slider` is the name
        // `role_of` turns into `SemanticRole::Slider`, so the copy loop drops it
        // and the accessibility tree reports one slider rather than two; `canvas`
        // maps to no role at all (`_ => SemanticRole::Group`), which is exactly
        // what an unnamed element mapped to before it had a name.
        let name_of = |set: &[obscur_view::view::ViewProperty]| {
            set.iter()
                .find(|p| p.name() == LOCAL_NAME)
                .map(|p| p.value().clone())
        };
        assert_eq!(
            name_of(bar),
            Some(ViewValue::Text("canvas".into())),
            "the bar is a canvas and says so, or no rule can size the painter's box"
        );
        assert_ne!(
            name_of(bar),
            name_of(props),
            "the host's name belongs to the host, or the canvas would claim the \
             role and the accessibility tree would report two sliders"
        );
    }

    #[test]
    fn the_scale_reaches_the_node_so_a_drag_can_read_it() {
        let slider = Slider::new(5.0).over(0.0, 20.0).stepped_by(5.0);
        let props = slider.props();
        assert_eq!(SliderScale::read(&props), slider.scale());
    }
}
