//! Resolves a text node's computed typography into shaping inputs:
//! [`attributes_for`] (weight + slant) and [`direction_for`] (base direction).

use obscur_style::computed::ComputedStyle;
use obscur_style::property::{layout_keyword, PropertyId, PropertyValue};
use obscur_text::font::FontAttributes;
use obscur_text::shaping::TextDirection;
use obscur_text::span::{FontSlant, FontWeight};

/// Builds the [`FontAttributes`] (weight + slant) for a text node from its
/// computed `font-weight` and `font-style` (§4.14, §5.10).
///
/// Reads [`PropertyId::FontWeight`] — a bare [`PropertyValue::Number`] on the CSS
/// `1..=1000` axis — into a [`FontWeight`] (clamped by [`FontWeight::new`]), and
/// [`PropertyId::FontStyle`] — a [`PropertyValue::Keyword`] backed by one of the
/// reserved [`layout_keyword`] handles — into a [`FontSlant`]
/// (`FONT_STYLE_ITALIC` → [`FontSlant::Italic`], `FONT_STYLE_OBLIQUE` →
/// [`FontSlant::Oblique`], everything else including an absent property →
/// [`FontSlant::Normal`]). When both properties are absent this returns
/// [`FontAttributes::default`] — byte-for-byte the behaviour before per-node
/// weight/style resolution.
///
/// This helper feeds **both** the layout measure/arrange path and the scene
/// compose path so the attributes a text box is measured with are the attributes
/// it is painted with.
#[must_use]
pub(crate) fn attributes_for(computed: Option<&ComputedStyle>) -> FontAttributes {
    let mut attributes = FontAttributes::default();
    let Some(style) = computed else {
        return attributes;
    };
    if let Some(value) = style.get(PropertyId::FontWeight)
        && let PropertyValue::Number(weight) = value.value()
    {
        attributes = attributes.with_weight(weight_from(*weight));
    }
    if let Some(value) = style.get(PropertyId::FontStyle)
        && let PropertyValue::Keyword(symbol) = value.value()
    {
        attributes = attributes.with_slant(slant_from(*symbol));
    }
    attributes
}

/// Narrows a CSS numeric weight into a [`FontWeight`] on the `1..=1000` axis.
///
/// The value is rounded and clamped to the representable range; [`FontWeight::new`]
/// clamps again, so an out-of-range author value never panics.
fn weight_from(weight: f32) -> FontWeight {
    // The weight is rounded to the nearest whole step and clamped into the
    // `1..=1000` axis before the (already-clamping) `FontWeight::new`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = weight.round().clamp(0.0, 1000.0) as u16;
    FontWeight::new(value)
}

/// Maps a reserved `font-style` keyword handle to its [`FontSlant`]; any other
/// symbol (including a stray interned keyword) resolves to [`FontSlant::Normal`].
fn slant_from(symbol: obscur_identity::InternedSymbol) -> FontSlant {
    if symbol == layout_keyword::FONT_STYLE_ITALIC {
        FontSlant::Italic
    } else if symbol == layout_keyword::FONT_STYLE_OBLIQUE {
        FontSlant::Oblique
    } else {
        FontSlant::Normal
    }
}

/// Resolves a text node's base [`TextDirection`] from its computed `direction`
/// (§4.14, §5.10).
///
/// Reads [`PropertyId::Direction`] — a [`PropertyValue::Keyword`] backed by one
/// of the reserved [`layout_keyword`] handles — returning
/// [`TextDirection::RightToLeft`] when it is `DIRECTION_RTL` and
/// [`TextDirection::LeftToRight`] otherwise (including `DIRECTION_LTR` and an
/// absent property). The itemizer uses this as the paragraph's base direction
/// and assigns bidi levels from it; the same value drives both the measure and
/// compose shaping passes so they agree.
#[must_use]
pub(crate) fn direction_for(computed: Option<&ComputedStyle>) -> TextDirection {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::Direction))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    if symbol == Some(layout_keyword::DIRECTION_RTL) {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::computed::ComputedValue;

    fn styled(property: PropertyId, value: PropertyValue) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            property,
            ComputedValue::from_concrete(value).expect("value is concrete"),
        );
        style
    }

    #[test]
    fn absent_typography_is_the_default_attributes() {
        assert_eq!(attributes_for(None), FontAttributes::default());
        let style = ComputedStyle::new();
        assert_eq!(attributes_for(Some(&style)), FontAttributes::default());
    }

    #[test]
    fn computed_weight_maps_to_the_font_weight() {
        let style = styled(PropertyId::FontWeight, PropertyValue::Number(700.0));
        assert_eq!(attributes_for(Some(&style)).weight, FontWeight::new(700));
    }

    #[test]
    fn computed_style_maps_to_the_slant() {
        let italic = styled(
            PropertyId::FontStyle,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_ITALIC),
        );
        assert_eq!(attributes_for(Some(&italic)).slant, FontSlant::Italic);
        let oblique = styled(
            PropertyId::FontStyle,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_OBLIQUE),
        );
        assert_eq!(attributes_for(Some(&oblique)).slant, FontSlant::Oblique);
        let normal = styled(
            PropertyId::FontStyle,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL),
        );
        assert_eq!(attributes_for(Some(&normal)).slant, FontSlant::Normal);
    }

    #[test]
    fn direction_defaults_to_ltr_and_flips_on_rtl() {
        assert_eq!(direction_for(None), TextDirection::LeftToRight);
        let ltr = styled(
            PropertyId::Direction,
            PropertyValue::Keyword(layout_keyword::DIRECTION_LTR),
        );
        assert_eq!(direction_for(Some(&ltr)), TextDirection::LeftToRight);
        let rtl = styled(
            PropertyId::Direction,
            PropertyValue::Keyword(layout_keyword::DIRECTION_RTL),
        );
        assert_eq!(direction_for(Some(&rtl)), TextDirection::RightToLeft);
    }
}
