#![doc = include_str!("grid_line_value.doc/_module.md")]

#[doc = include_str!("grid_line_value.doc/GridLineValue.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridLineValue {
    #[doc = include_str!("grid_line_value.doc/Auto.md")]
    Auto,
    #[doc = include_str!("grid_line_value.doc/Line.md")]
    Line(i32),
    #[doc = include_str!("grid_line_value.doc/Span.md")]
    Span(u16),
    #[doc = include_str!("grid_line_value.doc/Named.md")]
    Named(Box<str>),
}

impl Default for GridLineValue {
    #[doc = include_str!("grid_line_value.doc/default.md")]
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
