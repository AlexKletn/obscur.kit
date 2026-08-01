#![doc = include_str!("grid_placement.doc/_module.md")]

use super::value_lowering::css_wide_keyword;
use crate::parser::ParsedValue;
use obscur_style::property::{GridLineValue, PropertyValue, layout_keyword};

#[doc = include_str!("grid_placement.doc/grid_line_edge.md")]
pub(super) fn grid_line_edge(group: &[ParsedValue]) -> Option<GridLineValue> {
    match group {
        [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("auto") => {
            Some(GridLineValue::Auto)
        }
        // A lone `<custom-ident>` (not `auto`, not a CSS-wide keyword) is a named
        // line, resolved against the container's named lines in the runtime. `span`
        // alone is not a valid edge, so exclude it (a bare `span` is not a line).
        [ParsedValue::Keyword(keyword)]
            if css_wide_keyword(keyword).is_none() && !keyword.eq_ignore_ascii_case("span") =>
        {
            Some(GridLineValue::Named((**keyword).into()))
        }
        [one] => grid_line_integer(one).map(GridLineValue::Line),
        // `span N` / `N span`: exactly one `span` keyword and one positive integer.
        [a, b] => {
            let span = grid_line_span(a, b).or_else(|| grid_line_span(b, a))?;
            Some(GridLineValue::Span(span))
        }
        _ => None,
    }
}

#[doc = include_str!("grid_placement.doc/grid_line_integer.md")]
#[allow(clippy::cast_possible_truncation)]
fn grid_line_integer(value: &ParsedValue) -> Option<i32> {
    match value {
        ParsedValue::Number(n) if n.fract() == 0.0 && *n != 0.0 => Some(*n as i32),
        _ => None,
    }
}

#[doc = include_str!("grid_placement.doc/grid_line_span.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn grid_line_span(keyword: &ParsedValue, count: &ParsedValue) -> Option<u16> {
    let ParsedValue::Keyword(keyword) = keyword else {
        return None;
    };
    if !keyword.eq_ignore_ascii_case("span") {
        return None;
    }
    match count {
        ParsedValue::Number(n) if n.fract() == 0.0 && *n >= 1.0 => {
            Some((*n as u32).min(u32::from(u16::MAX)) as u16)
        }
        _ => None,
    }
}

#[doc = include_str!("grid_placement.doc/lower_grid_line.md")]
pub(super) fn lower_grid_line(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return Some(wide);
    }
    grid_line_edge(values).map(PropertyValue::GridLine)
}

#[doc = include_str!("grid_placement.doc/lower_grid_auto_flow.md")]
pub(super) fn lower_grid_auto_flow(values: &[ParsedValue]) -> Option<PropertyValue> {
    // A lone CSS-wide keyword (`inherit` / `initial` / `unset`) wins.
    if let [ParsedValue::Keyword(keyword)] = values
        && let Some(wide) = css_wide_keyword(keyword)
    {
        return Some(wide);
    }
    // Otherwise the value is one or two bare axis / `dense` keywords.
    let mut tokens: Vec<&str> = Vec::new();
    for value in values {
        match value {
            ParsedValue::Keyword(keyword) => tokens.push(keyword),
            _ => return None,
        }
    }
    let joined = tokens.join(" ");
    layout_keyword::grid_auto_flow_from_name(&joined).map(PropertyValue::Keyword)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::longhand_catalog::property_id_from_name;
    use super::super::property_compiler::tests::{
        compile, compile_all, standard, standard_of, value_of,
    };

    use crate::compiler::CompiledDeclaration;

    use obscur_style::property::{GridLineValue, PropertyId, PropertyValue, layout_keyword};

    #[doc = include_str!("grid_placement.doc/grid_lines.md")]
    pub(crate) fn grid_lines(compiled: &[CompiledDeclaration]) -> Vec<(PropertyId, GridLineValue)> {
        compiled
            .iter()
            .map(|d| {
                let decl = standard_of(d);
                let PropertyValue::GridLine(edge) = decl.value() else {
                    panic!("expected grid line value, got {:?}", decl.value());
                };
                (decl.property(), edge.clone())
            })
            .collect()
    }

    #[test]
    fn grid_line_auto_edge_is_accepted() {
        assert_eq!(
            grid_lines(&compile_all("grid-column: auto / 3")),
            vec![
                (PropertyId::GridColumnStart, GridLineValue::Auto),
                (PropertyId::GridColumnEnd, GridLineValue::Line(3)),
            ]
        );
    }

    #[test]
    fn grid_line_longhands_lower_directly() {
        assert_eq!(
            value_of("grid-column-start: 1"),
            PropertyValue::GridLine(GridLineValue::Line(1))
        );
        assert_eq!(
            value_of("grid-column-end: span 3"),
            PropertyValue::GridLine(GridLineValue::Span(3))
        );
        assert_eq!(
            value_of("grid-row-start: auto"),
            PropertyValue::GridLine(GridLineValue::Auto)
        );
        assert_eq!(
            standard("grid-row-end: 4").0.property(),
            PropertyId::GridRowEnd
        );
    }

    #[test]
    fn grid_line_longhand_honours_css_wide_keywords() {
        assert_eq!(
            value_of("grid-column-start: inherit"),
            PropertyValue::Inherit
        );
        assert_eq!(value_of("grid-row-end: initial"), PropertyValue::Initial);
    }

    #[test]
    fn invalid_grid_line_is_unsupported() {
        // A bare length is not a valid grid line (an integer, `span N`, `auto`, or
        // a `<custom-ident>` name are; a `px` value is not).
        let decls = compile_all("grid-column: 50px");
        assert_eq!(decls.len(), 1);
        assert!(matches!(decls[0], CompiledDeclaration::Unsupported(_)));
        // A zero line index is invalid.
        let (result, _) = compile("grid-column-start: 0");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
        // More than one `/` is not a valid placement.
        let decls = compile_all("grid-column: 1 / 2 / 3");
        assert_eq!(decls.len(), 1);
        assert!(matches!(decls[0], CompiledDeclaration::Unsupported(_)));
    }

    // ---- grid-area shorthand ----

    #[test]
    fn grid_auto_flow_maps_to_reserved_symbols() {
        assert_eq!(
            property_id_from_name("grid-auto-flow"),
            Some(PropertyId::GridAutoFlow)
        );
        assert_eq!(
            value_of("grid-auto-flow: row"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_ROW)
        );
        assert_eq!(
            value_of("grid-auto-flow: column"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_COLUMN)
        );
        // Bare `dense` folds to row-dense.
        assert_eq!(
            value_of("grid-auto-flow: dense"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_ROW_DENSE)
        );
        // The multi-token `column dense` folds to a single canonical symbol.
        assert_eq!(
            value_of("grid-auto-flow: column dense"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE)
        );
        assert_eq!(
            value_of("grid-auto-flow: row dense"),
            PropertyValue::Keyword(layout_keyword::GRID_AUTO_FLOW_ROW_DENSE)
        );
        assert_eq!(
            standard("grid-auto-flow: dense").0.property(),
            PropertyId::GridAutoFlow
        );
    }
}
