#![doc = include_str!("../../docs/api/computed/relative_font_weight.doc/_module.md")]

use crate::computed::ComputedStyle;
use crate::property::{PropertyId, PropertyValue, layout_keyword};
use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/computed/relative_font_weight.doc/resolve_relative_weight.md")]
pub(super) fn resolve_relative_weight(
    value: &PropertyValue,
    parent: Option<&ComputedStyle>,
) -> Option<f32> {
    let PropertyValue::Keyword(symbol) = value else {
        return None;
    };
    let bolder = relative_weight_is_bolder(*symbol)?;
    Some(resolve_relative_font_weight(
        parent_font_weight(parent),
        bolder,
    ))
}

/// The CSS initial `font-weight` (`normal`), and the weight a relative
/// `bolder` / `lighter` resolves against when there is no parent weight.
const NORMAL_FONT_WEIGHT: f32 = 400.0;

/// Classifies a reserved relative-`font-weight` handle: `Some(true)` for
/// `bolder`, `Some(false)` for `lighter`, and [`None`] for any other symbol.
fn relative_weight_is_bolder(symbol: InternedSymbol) -> Option<bool> {
    if symbol == layout_keyword::FONT_WEIGHT_BOLDER {
        Some(true)
    } else if symbol == layout_keyword::FONT_WEIGHT_LIGHTER {
        Some(false)
    } else {
        None
    }
}

/// Reads the parent's computed numeric `font-weight`, defaulting to
/// [`NORMAL_FONT_WEIGHT`] (400) when there is no parent or the parent did not
/// compute a numeric weight (the root's implicit parent is `normal`).
fn parent_font_weight(parent: Option<&ComputedStyle>) -> f32 {
    parent
        .and_then(|p| p.get(PropertyId::FontWeight))
        .and_then(|value| match value.value() {
            PropertyValue::Number(weight) => Some(*weight),
            _ => None,
        })
        .unwrap_or(NORMAL_FONT_WEIGHT)
}

#[doc = include_str!("../../docs/api/computed/relative_font_weight.doc/resolve_relative_font_weight.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn resolve_relative_font_weight(parent_weight: f32, bolder: bool) -> f32 {
    let bucket = (parent_weight / 100.0).round().clamp(1.0, 9.0) as u16;
    let (heavier, lighter) = match bucket {
        1..=3 => (400u16, 100u16),
        4 | 5 => (700, 100),
        6 | 7 => (900, 400),
        // 8..=9 (the clamp guarantees the bucket is within `1..=9`).
        _ => (900, 700),
    };
    f32::from(if bolder { heavier } else { lighter })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_font_weight_table() {
        // The nine CSS buckets, keyed on the parent's computed weight.
        assert_eq!(resolve_relative_font_weight(100.0, true), 400.0);
        assert_eq!(resolve_relative_font_weight(100.0, false), 100.0);
        assert_eq!(resolve_relative_font_weight(400.0, true), 700.0);
        assert_eq!(resolve_relative_font_weight(400.0, false), 100.0);
        assert_eq!(resolve_relative_font_weight(500.0, true), 700.0);
        assert_eq!(resolve_relative_font_weight(600.0, true), 900.0);
        assert_eq!(resolve_relative_font_weight(600.0, false), 400.0);
        assert_eq!(resolve_relative_font_weight(700.0, true), 900.0);
        assert_eq!(resolve_relative_font_weight(700.0, false), 400.0);
        assert_eq!(resolve_relative_font_weight(800.0, false), 700.0);
        assert_eq!(resolve_relative_font_weight(900.0, false), 700.0);
        // In-between values round to the nearest hundred bucket.
        assert_eq!(resolve_relative_font_weight(430.0, true), 700.0);
        // Out-of-range weights clamp into `100..=900`.
        assert_eq!(resolve_relative_font_weight(20.0, true), 400.0);
        assert_eq!(resolve_relative_font_weight(1500.0, false), 700.0);
    }
}
