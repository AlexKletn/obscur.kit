#![doc = include_str!("../../docs/api/progress/progress.doc/_module.md")]

use super::ProgressShape;

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Canvas;
use obscur_view::view::{
    LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue, canvas_painter_prop,
};

/// A progress bar.
#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    /// How far along, from 0 to 1, clamped at construction.
    value: f32,
    /// Whether the bar knows how far along it is.
    tracked: bool,
    /// The shape the bar is drawn along.
    shape: ProgressShape,
    /// SVG path data the bar runs along, when the author supplied one.
    path: Option<Box<str>>,
}

impl Progress {
    /// The component id an authored `<progress>` binds to.
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0007);

    /// The built-in painter every progress names.
    pub const PAINTER: InternedSymbol = InternedSymbol::from_raw(0x00D1_0701);

    /// The fraction, as the painter reads it.
    pub const PROP_VALUE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0702);
    /// The shape, as the painter reads it.
    pub const PROP_SHAPE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0703);
    /// Whether the bar knows how far along it is.
    pub const PROP_TRACKED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0704);
    /// SVG path data the bar runs along, in the canvas's own coordinates.
    pub const PROP_PATH: InternedSymbol = InternedSymbol::from_raw(0x00D1_0705);

    /// A progress bar at `value`, clamped to 0..1; a non-finite value reads as zero.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self {
            // Clamped at the door: every reader downstream — the painter, the
            // accessibility projection — would otherwise have to clamp again,
            // and one of them would forget.
            value: if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            },
            shape: ProgressShape::Line,
            path: None,
            tracked: true,
        }
    }

    #[doc = include_str!("../../docs/api/progress/progress.doc/along.md")]
    #[must_use]
    pub fn along(mut self, path_data: impl Into<Box<str>>) -> Self {
        self.path = Some(path_data.into());
        self
    }

    #[doc = include_str!("../../docs/api/progress/progress.doc/indeterminate.md")]
    #[must_use]
    pub fn indeterminate() -> Self {
        Self {
            value: 0.0,
            shape: ProgressShape::Line,
            path: None,
            tracked: false,
        }
    }

    /// Whether the bar knows how far along it is.
    #[must_use]
    pub const fn is_tracked(&self) -> bool {
        self.tracked
    }

    /// Returns the bar drawn along `shape`.
    #[must_use]
    pub fn shaped(mut self, shape: ProgressShape) -> Self {
        self.shape = shape;
        self
    }

    /// How far along, from 0 to 1.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// The host node's properties: its local name, and what the painter reads.
    #[must_use]
    pub fn props(&self) -> Vec<ViewProperty> {
        vec![
            ViewProperty::new(LOCAL_NAME, ViewValue::Text("progress".into())),
            ViewProperty::new(Self::PROP_VALUE, ViewValue::Float(f64::from(self.value))),
            ViewProperty::new(Self::PROP_SHAPE, ViewValue::Int(self.shape.as_int())),
            ViewProperty::new(Self::PROP_TRACKED, ViewValue::Bool(self.tracked)),
        ]
    }

    #[doc = include_str!("../../docs/api/progress/progress.doc/contents.md")]
    #[must_use]
    pub fn contents(
        value: f32,
        shape: ProgressShape,
        tracked: bool,
        path: Option<&str>,
    ) -> ViewFragment {
        // The bar is a `<canvas>` that names the built-in painter and carries
        // what the painter needs. The widget draws nothing itself: one painter
        // serves every progress on the screen, and each instance parameterises
        // it through these two properties.
        ViewFragment::from_nodes(vec![
            Canvas::new(0.0, 0.0)
                .into_view_node()
                .with_prop(canvas_painter_prop(Self::PAINTER))
                .with_prop(ViewProperty::new(
                    Self::PROP_VALUE,
                    ViewValue::Float(f64::from(value)),
                ))
                .with_prop(ViewProperty::new(
                    Self::PROP_SHAPE,
                    ViewValue::Int(shape.as_int()),
                ))
                .with_prop(ViewProperty::new(
                    Self::PROP_TRACKED,
                    ViewValue::Bool(tracked),
                ))
                .with_prop(ViewProperty::new(
                    Self::PROP_PATH,
                    ViewValue::Text(path.unwrap_or("").into()),
                )),
        ])
    }

    /// The bar as a view.
    #[must_use]
    pub fn view(&self) -> View {
        let mut node = ViewNode::element(Self::COMPONENT_ID);
        for prop in self.props() {
            node = node.with_prop(prop);
        }
        View::single(node.with_children(Self::contents(
            self.value,
            self.shape,
            self.tracked,
            self.path.as_deref(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::{Progress, ProgressShape};
    use obscur_view::view::{ViewNode, ViewValue, canvas_painter};

    #[test]
    fn a_value_is_clamped_at_the_door() {
        // Once, here, rather than in every reader downstream — one of which
        // would forget, and a ring drawn at 1.4 would wrap past its own start.
        assert_eq!(Progress::new(1.5).value(), 1.0);
        assert_eq!(Progress::new(-2.0).value(), 0.0);
        assert_eq!(Progress::new(f32::NAN).value(), 0.0);
        assert_eq!(Progress::new(0.25).value(), 0.25);
    }

    #[test]
    fn the_bar_is_a_canvas_naming_the_built_in_painter() {
        let view = Progress::new(0.5).shaped(ProgressShape::Ring).view();
        let ViewNode::Element { children, .. } = &view.root().nodes()[0] else {
            panic!("a progress is an element");
        };
        let ViewNode::Element { props, .. } = &children.nodes()[0] else {
            panic!("its bar is an element");
        };
        assert_eq!(canvas_painter(props), Some(Progress::PAINTER));
        assert!(
            props
                .iter()
                .any(|p| p.name() == Progress::PROP_VALUE && *p.value() == ViewValue::Float(0.5)),
            "the value the painter reads rides on the canvas it draws"
        );
        assert!(
            props
                .iter()
                .any(|p| p.name() == Progress::PROP_SHAPE && *p.value() == ViewValue::Int(1)),
            "and so does the shape"
        );
    }

    #[test]
    fn the_host_carries_the_local_name_the_role_is_derived_from() {
        // `progress` maps to `SemanticRole::ProgressBar`, which is deliberately
        // NOT interactive — so Tab passes it by, which is what a bar that
        // reports rather than accepts should do.
        let view = Progress::new(0.1).view();
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("a progress is an element");
        };
        assert!(props.iter().any(|p| {
            p.name() == obscur_view::view::LOCAL_NAME
                && *p.value() == ViewValue::Text("progress".into())
        }));
    }

    #[test]
    fn an_unmodelled_shape_reads_as_a_line_rather_than_as_nothing() {
        assert_eq!(ProgressShape::from_int(7), ProgressShape::Line);
        assert_eq!(ProgressShape::from_int(1), ProgressShape::Ring);
        assert_eq!(
            ProgressShape::from_int(ProgressShape::Ring.as_int()),
            ProgressShape::Ring,
            "the pair round-trips"
        );
    }
}
