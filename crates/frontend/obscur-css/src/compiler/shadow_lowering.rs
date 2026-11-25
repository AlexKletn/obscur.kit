#![doc = include_str!("../../docs/api/compiler/shadow_lowering.doc/_module.md")]

use super::color_lowering::parse_stop_color;
use super::value_lowering::dimension_length;
use crate::parser::ParsedValue;
use obscur_style::property::{BoxShadow, Length, PropertyValue, StyleColor};

#[doc = include_str!("../../docs/api/compiler/shadow_lowering.doc/lower_box_shadow.md")]
pub(super) fn lower_box_shadow(values: &[ParsedValue]) -> Option<PropertyValue> {
    // `none` → an empty shadow list.
    if let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::BoxShadow(Box::new([])));
    }
    let mut shadows: Vec<BoxShadow> = Vec::new();
    for layer in values.split(|value| matches!(value, ParsedValue::Comma)) {
        if layer.is_empty() {
            // A stray / trailing comma leaves an empty layer, which is invalid.
            return None;
        }
        shadows.push(lower_one_shadow(layer)?);
    }
    if shadows.is_empty() {
        return None;
    }
    Some(PropertyValue::BoxShadow(shadows.into_boxed_slice()))
}

#[doc = include_str!("../../docs/api/compiler/shadow_lowering.doc/lower_one_shadow.md")]
fn lower_one_shadow(layer: &[ParsedValue]) -> Option<BoxShadow> {
    let mut lengths: Vec<Length> = Vec::new();
    let mut color: Option<StyleColor> = None;
    let mut inset = false;
    for value in layer {
        // The `inset` keyword (in any position) takes priority over reading it as
        // a color (`inset` is not a named color, but this keeps the intent clear).
        if let ParsedValue::Keyword(keyword) = value
            && keyword.eq_ignore_ascii_case("inset")
        {
            if inset {
                return None;
            }
            inset = true;
            continue;
        }
        if let Some(length) = shadow_length(value) {
            if lengths.len() == 4 {
                return None;
            }
            lengths.push(length);
            continue;
        }
        if let Some(parsed) = parse_stop_color(value) {
            if color.is_some() {
                return None;
            }
            color = Some(parsed);
            continue;
        }
        return None;
    }
    // Two required offsets, then optional blur and spread; the `Length::px(0.0)`
    // fills match the CSS initial blur / spread.
    let (offset_x, offset_y, blur, spread) = match lengths.as_slice() {
        [x, y] => (*x, *y, Length::px(0.0), Length::px(0.0)),
        [x, y, b] => (*x, *y, *b, Length::px(0.0)),
        [x, y, b, s] => (*x, *y, *b, *s),
        _ => return None,
    };
    Some(BoxShadow::new(
        offset_x, offset_y, blur, spread, color, inset,
    ))
}

/// Lowers one `box-shadow` length component: a `<length>` (`px`, `em` / `rem`,
/// the viewport units) or a unitless `0` (`0px`). Percentages and keywords are not
/// `box-shadow` lengths and yield [`None`].
pub(super) fn shadow_length(value: &ParsedValue) -> Option<Length> {
    match value {
        ParsedValue::Number(n) if *n == 0.0 => Some(Length::px(0.0)),
        ParsedValue::Dimension { value, unit } => dimension_length(*value, unit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::color_lowering::tests::color;
    use super::super::property_compiler::tests::value_of;

    use obscur_style::property::{BoxShadow, Length, PropertyValue, StyleColor};

    /// Extracts the lowered `box-shadow` list from a declaration.
    fn shadows_of(text: &str) -> Box<[BoxShadow]> {
        match value_of(text) {
            PropertyValue::BoxShadow(list) => list,
            other => panic!("expected a box-shadow list, got {other:?}"),
        }
    }

    #[test]
    fn box_shadow_drop_offset_blur_color() {
        // `2px 4px 6px red` → offset (2,4), blur 6, spread 0, red, not inset.
        let list = shadows_of("box-shadow: 2px 4px 6px red");
        assert_eq!(list.len(), 1);
        let s = &list[0];
        assert_eq!(s.offset_x(), Length::px(2.0));
        assert_eq!(s.offset_y(), Length::px(4.0));
        assert_eq!(s.blur(), Length::px(6.0));
        assert_eq!(s.spread(), Length::px(0.0));
        assert_eq!(s.color(), Some(color(1.0, 0.0, 0.0)));
        assert!(!s.is_inset());
    }

    #[test]
    fn box_shadow_inset_with_spread_and_currentcolor_default() {
        // `0px 0px 0px 2px #000 inset` → spread 2, inset; explicit black color.
        let list = shadows_of("box-shadow: 0px 0px 0px 2px #000 inset");
        assert_eq!(list.len(), 1);
        let s = &list[0];
        assert_eq!(s.offset_x(), Length::px(0.0));
        assert_eq!(s.offset_y(), Length::px(0.0));
        assert_eq!(s.blur(), Length::px(0.0));
        assert_eq!(s.spread(), Length::px(2.0));
        assert_eq!(s.color(), Some(StyleColor::BLACK));
        assert!(s.is_inset());

        // With no authored color the shadow keeps the `currentColor` default.
        let default = shadows_of("box-shadow: 1px 1px");
        assert_eq!(default[0].color(), None);
    }

    #[test]
    fn box_shadow_comma_separated_list_preserves_order() {
        let list = shadows_of("box-shadow: 1px 1px 2px red, 0px 0px 0px 3px blue inset");
        assert_eq!(list.len(), 2);
        // First-listed shadow.
        assert_eq!(list[0].offset_x(), Length::px(1.0));
        assert_eq!(list[0].color(), Some(color(1.0, 0.0, 0.0)));
        assert!(!list[0].is_inset());
        // Second-listed shadow.
        assert_eq!(list[1].spread(), Length::px(3.0));
        assert_eq!(list[1].color(), Some(color(0.0, 0.0, 1.0)));
        assert!(list[1].is_inset());
    }

    #[test]
    fn box_shadow_none_is_an_empty_list() {
        let list = shadows_of("box-shadow: none");
        assert!(list.is_empty());
    }
}
