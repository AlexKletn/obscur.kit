//! `GridLineValue`: one style-local edge of a grid item's line placement.

#[doc = include_str!("../../docs/api/property/grid_line_value.doc/GridLineValue.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridLineValue {
    /// No explicit line on this edge; the item is auto-placed here.
    Auto,
    /// A 1-based grid line index.
    Line(i32),
    /// `span N`: this edge is `N` tracks from the opposite edge.
    Span(u16),
    /// A `<custom-ident>` line name, resolved against the container's named lines
    /// in the runtime (spec §4.13).
    Named(Box<str>),
    /// A nonzero occurrence of a line name, counted from the end if negative.
    NamedOccurrence(Box<str>, std::num::NonZeroI32),
}

impl Default for GridLineValue {
    /// The initial value: [`Auto`](GridLineValue::Auto) (pure auto-placement).
    fn default() -> Self {
        GridLineValue::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::GridLineValue;

    #[test]
    fn default_is_auto() {
        assert_eq!(GridLineValue::default(), GridLineValue::Auto);
    }

    #[test]
    fn variants_carry_their_payloads() {
        assert_eq!(GridLineValue::Line(3), GridLineValue::Line(3));
        assert_eq!(GridLineValue::Span(2), GridLineValue::Span(2));
        assert_ne!(GridLineValue::Line(1), GridLineValue::Span(1));
        assert_eq!(
            GridLineValue::Named("main".into()),
            GridLineValue::Named("main".into())
        );
        assert_ne!(
            GridLineValue::Named("main".into()),
            GridLineValue::Named("side".into())
        );
    }
}
