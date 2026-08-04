#![doc = include_str!("canvas.doc/_module.md")]

use obscur_identity::InternedSymbol;

use super::{ViewProperty, ViewValue};

#[doc = include_str!("canvas.doc/CanvasPaint.md")]
#[derive(Debug, Clone, Copy)]
pub struct CanvasPaint<'a> {
    #[doc = include_str!("canvas.doc/paint_props.md")]
    props: &'a [ViewProperty],
    #[doc = include_str!("canvas.doc/paint_elapsed.md")]
    elapsed: f32,
}

impl<'a> CanvasPaint<'a> {
    #[doc = include_str!("canvas.doc/paint_new.md")]
    #[must_use]
    pub const fn new(props: &'a [ViewProperty], elapsed: f32) -> Self {
        Self { props, elapsed }
    }

    #[doc = include_str!("canvas.doc/paint_props_2.md")]
    #[must_use]
    pub const fn props(&self) -> &'a [ViewProperty] {
        self.props
    }

    #[doc = include_str!("canvas.doc/paint_elapsed_2.md")]
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

#[doc = include_str!("canvas.doc/CANVAS_PAINTER.md")]
pub const CANVAS_PAINTER: InternedSymbol = InternedSymbol::from_raw(0x0000_0401);

#[doc = include_str!("canvas.doc/canvas_painter.md")]
#[must_use]
pub fn canvas_painter(props: &[ViewProperty]) -> Option<InternedSymbol> {
    props.iter().find_map(|prop| {
        (prop.name() == CANVAS_PAINTER).then(|| match prop.value() {
            ViewValue::Int(raw) => u32::try_from(*raw).ok().map(InternedSymbol::from_raw),
            _ => None,
        })?
    })
}

#[doc = include_str!("canvas.doc/canvas_painter_prop.md")]
#[must_use]
pub fn canvas_painter_prop(painter: InternedSymbol) -> ViewProperty {
    ViewProperty::new(CANVAS_PAINTER, ViewValue::Int(i64::from(painter.raw())))
}

#[cfg(test)]
mod tests {
    use super::{CANVAS_PAINTER, canvas_painter, canvas_painter_prop};
    use crate::view::{ViewProperty, ViewValue};
    use obscur_identity::InternedSymbol;

    const CHART: InternedSymbol = InternedSymbol::from_raw(0xABCD);

    #[test]
    fn a_prop_round_trips_through_the_reader() {
        assert_eq!(
            canvas_painter(&[canvas_painter_prop(CHART)]),
            Some(CHART),
            "what the writer emits is what the reader finds — the two halves of \
             one join, which is where this kind of binding goes wrong"
        );
    }

    #[test]
    fn a_node_naming_no_painter_answers_none() {
        assert_eq!(canvas_painter(&[]), None);
        assert_eq!(
            canvas_painter(&[ViewProperty::new(
                InternedSymbol::from_raw(0x0000_0001),
                ViewValue::Int(5)
            )]),
            None,
            "another property is not this one"
        );
    }

    #[test]
    fn a_wrongly_typed_value_answers_none_rather_than_guessing() {
        assert_eq!(
            canvas_painter(&[ViewProperty::new(
                CANVAS_PAINTER,
                ViewValue::Text("chart".into())
            )]),
            None
        );
        assert_eq!(
            canvas_painter(&[ViewProperty::new(CANVAS_PAINTER, ViewValue::Int(-1))]),
            None,
            "a symbol is a u32; a negative integer is not one"
        );
    }

    #[test]
    fn the_symbol_is_outside_every_other_reserved_range() {
        // Style attributes are `0x0000_00xx`, event names `0x0000_01xx`, the slot
        // pair `0x0000_030x`. A collision would make one property answer two
        // questions.
        assert_ne!(CANVAS_PAINTER, super::super::CLASS);
        assert_ne!(CANVAS_PAINTER, super::super::EVENT_CLICK);
        assert_ne!(CANVAS_PAINTER, super::super::SLOT);
        assert_eq!(CANVAS_PAINTER.raw() & 0xFFFF_FF00, 0x0000_0400);
    }
}
