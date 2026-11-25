#![doc = include_str!("../../docs/api/view/canvas.doc/_module.md")]

use obscur_diagnostics::Diagnostic;
use obscur_identity::InternedSymbol;

use super::{ViewProperty, ViewValue};

#[doc = include_str!("../../docs/api/view/canvas.doc/CanvasPaint.md")]
#[derive(Clone, Copy)]
pub struct CanvasPaint<'a> {
    /// The properties of the node being drawn.
    props: &'a [ViewProperty],
    /// Seconds since the host started driving the clock.
    elapsed: f32,
    /// Optional diagnostic callback supplied by the host.
    report: Option<&'a (dyn Fn(Diagnostic) + Sync)>,
}

impl std::fmt::Debug for CanvasPaint<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasPaint")
            .field("props", &self.props)
            .field("elapsed", &self.elapsed)
            .finish_non_exhaustive()
    }
}

impl<'a> CanvasPaint<'a> {
    /// The context for one node's paint.
    #[must_use]
    pub const fn new(props: &'a [ViewProperty], elapsed: f32) -> Self {
        Self {
            props,
            elapsed,
            report: None,
        }
    }

    /// Installs a callback for diagnostics emitted by this paint invocation.
    #[must_use]
    pub fn with_report_handler(mut self, report: &'a (dyn Fn(Diagnostic) + Sync)) -> Self {
        self.report = Some(report);
        self
    }

    /// Reports a diagnostic to the host when a callback is installed.
    pub fn report(&self, diagnostic: Diagnostic) {
        if let Some(report) = self.report {
            report(diagnostic);
        }
    }

    /// The properties of the node being drawn — one painter serves every node that names it, and this is how it tells them apart.
    #[must_use]
    pub const fn props(&self) -> &'a [ViewProperty] {
        self.props
    }

    /// Seconds since the host started driving the clock, for a canvas that draws a moving thing.
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

/// The reserved property naming a node's painter.
pub const CANVAS_PAINTER: InternedSymbol = InternedSymbol::from_raw(0x0000_0401);

/// The painter a node names, or [`None`] when it names none or names it wrongly.
#[must_use]
pub fn canvas_painter(props: &[ViewProperty]) -> Option<InternedSymbol> {
    props.iter().find_map(|prop| {
        (prop.name() == CANVAS_PAINTER).then(|| match prop.value() {
            ViewValue::Int(raw) => u32::try_from(*raw).ok().map(InternedSymbol::from_raw),
            _ => None,
        })?
    })
}

/// The property that names `painter` — the writing half of the join [`canvas_painter`] reads.
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
