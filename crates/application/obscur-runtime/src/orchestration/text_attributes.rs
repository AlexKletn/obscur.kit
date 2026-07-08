//! Resolves a text node's computed typography into shaping inputs:
//! [`attributes_for`] (weight + slant) and [`direction_for`] (base direction).

use std::borrow::Cow;

use obscur_style::computed::ComputedStyle;
use obscur_style::property::{layout_keyword, PropertyId, PropertyValue};
use obscur_text::font::FontAttributes;
use obscur_text::layout::{TextAlignment, TextOverflow};
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

/// Resolves a text node's line [`TextAlignment`] from its computed `text-align`
/// (§4.14, §5.10).
///
/// Reads [`PropertyId::TextAlign`] — a [`PropertyValue::Keyword`] backed by one of
/// the reserved [`layout_keyword`] handles — mapping each `TEXT_ALIGN_*` handle to
/// its [`TextAlignment`] variant. An absent property (or any unrecognised keyword)
/// resolves to [`TextAlignment::Start`], byte-for-byte the hardcoded behaviour
/// before per-node alignment resolution. The same value drives both the layout
/// measure/arrange pass and the scene compose pass so the sized box and the
/// painted lines agree.
#[must_use]
pub(crate) fn alignment_for(computed: Option<&ComputedStyle>) -> TextAlignment {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::TextAlign))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    match symbol {
        Some(s) if s == layout_keyword::TEXT_ALIGN_LEFT => TextAlignment::Left,
        Some(s) if s == layout_keyword::TEXT_ALIGN_RIGHT => TextAlignment::Right,
        Some(s) if s == layout_keyword::TEXT_ALIGN_CENTER => TextAlignment::Center,
        Some(s) if s == layout_keyword::TEXT_ALIGN_JUSTIFY => TextAlignment::Justify,
        Some(s) if s == layout_keyword::TEXT_ALIGN_END => TextAlignment::End,
        // `start` and everything else (including an absent property) → Start.
        _ => TextAlignment::Start,
    }
}

/// Resolves a text node's [`TextOverflow`] policy from its computed
/// `text-overflow` (§4.14, §5.10).
///
/// Reads [`PropertyId::TextOverflow`] — a [`PropertyValue::Keyword`] backed by one
/// of the reserved [`layout_keyword`] handles — mapping `TEXT_OVERFLOW_ELLIPSIS`
/// to [`TextOverflow::Ellipsis`] and everything else (including `TEXT_OVERFLOW_CLIP`
/// and an absent property) to [`TextOverflow::Clip`], the CSS initial. The same
/// value drives both the layout measure/arrange pass and the scene compose pass so
/// the sized box and the painted (possibly truncated) lines agree.
#[must_use]
pub(crate) fn overflow_for(computed: Option<&ComputedStyle>) -> TextOverflow {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::TextOverflow))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    if symbol == Some(layout_keyword::TEXT_OVERFLOW_ELLIPSIS) {
        TextOverflow::Ellipsis
    } else {
        TextOverflow::Clip
    }
}

/// Resolves a text node's authored `line-height` into a logical-pixel line box
/// height from its computed `line-height` and `font_size` (§4.14, §5.10).
///
/// Reads [`PropertyId::LineHeight`], returning:
/// * `Some(multiple * font_size)` for a unitless [`PropertyValue::Number`]
///   (a multiple of the font size, the common `line-height: 1.5` form);
/// * `Some(px)` for a [`PropertyValue::Length`] in absolute pixels
///   (`line-height: 24px`);
/// * [`None`] for the initial `normal` keyword, an absent property, a relative
///   length unit, or any other value — the caller then falls back to the font's
///   natural (metric) line height, keeping the default path byte-for-byte
///   identical to pre-`line-height` behaviour.
///
/// The same value drives both the layout measure/arrange pass and the scene
/// compose pass, so the sized box height and the painted line spacing agree.
#[must_use]
pub(crate) fn line_height_for(computed: Option<&ComputedStyle>, font_size: f32) -> Option<f32> {
    match computed?.get(PropertyId::LineHeight)?.value() {
        PropertyValue::Number(multiple) => Some(multiple * font_size),
        PropertyValue::Length(length) => length.as_px().map(|px| px.0),
        _ => None,
    }
}

/// Resolves a text node's `letter-spacing` into extra per-cluster tracking, in
/// logical pixels, from its computed `letter-spacing` (CSS Text 3 §8.1).
///
/// Reads [`PropertyId::LetterSpacing`]: an absolute [`PropertyValue::Length`]
/// resolves to its pixel value (which may be negative); the initial `normal`
/// keyword, an absent property, a non-pixel length unit (`em` etc., not resolved
/// here — a documented refinement), or any other value resolves to `0.0`, so an
/// unauthored text node adds no tracking and shapes byte-for-byte as before.
///
/// The same value feeds **both** the layout measure/arrange pass and the scene
/// compose pass (through the shared [`apply_spacing`](obscur_text::shaping::apply_spacing)
/// seam) so the measured box width and the painted advances agree.
#[must_use]
pub(crate) fn letter_spacing_for(computed: Option<&ComputedStyle>) -> f32 {
    length_px_of(computed, PropertyId::LetterSpacing)
}

/// Resolves a text node's `word-spacing` into extra per-separator spacing, in
/// logical pixels, from its computed `word-spacing` (CSS Text 3 §8.2).
///
/// Reads [`PropertyId::WordSpacing`] with the same rules as
/// [`letter_spacing_for`]: an absolute pixel length (possibly negative) resolves
/// verbatim; `normal` / absent / non-pixel / other → `0.0`. Fed to the shared
/// spacing seam on both the measure and compose passes so they cannot drift.
#[must_use]
pub(crate) fn word_spacing_for(computed: Option<&ComputedStyle>) -> f32 {
    length_px_of(computed, PropertyId::WordSpacing)
}

/// Reads `property` off `computed` as an absolute-pixel [`PropertyValue::Length`],
/// or `0.0` for an absent property, a non-length value, or a non-pixel unit.
fn length_px_of(computed: Option<&ComputedStyle>, property: PropertyId) -> f32 {
    let Some(value) = computed.and_then(|style| style.get(property)) else {
        return 0.0;
    };
    match value.value() {
        PropertyValue::Length(length) => length.as_px().map_or(0.0, |px| px.0),
        _ => 0.0,
    }
}

/// The CSS Text 3 `text-transform` casing operation (§2.1).
///
/// Only the closed case-mapping keyword set is modelled; `full-width` /
/// `full-size-kana` are documented omissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextTransform {
    /// `none` (the initial): the text is shaped unchanged.
    None,
    /// `uppercase`: every character is mapped to uppercase.
    Uppercase,
    /// `lowercase`: every character is mapped to lowercase.
    Lowercase,
    /// `capitalize`: the first typographic letter of each word is uppercased.
    Capitalize,
}

/// Resolves a text node's [`TextTransform`] from its computed `text-transform`
/// (CSS Text 3 §2.1).
///
/// Reads [`PropertyId::TextTransform`] — a [`PropertyValue::Keyword`] backed by
/// one of the reserved [`layout_keyword`] handles — mapping each
/// `TEXT_TRANSFORM_*` handle to its variant. An absent property (or any
/// unrecognised keyword) resolves to [`TextTransform::None`], so an unauthored
/// text node's string is shaped unchanged.
#[must_use]
pub(crate) fn text_transform_for(computed: Option<&ComputedStyle>) -> TextTransform {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::TextTransform))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    match symbol {
        Some(s) if s == layout_keyword::TEXT_TRANSFORM_UPPERCASE => TextTransform::Uppercase,
        Some(s) if s == layout_keyword::TEXT_TRANSFORM_LOWERCASE => TextTransform::Lowercase,
        Some(s) if s == layout_keyword::TEXT_TRANSFORM_CAPITALIZE => TextTransform::Capitalize,
        // `none` and everything else (including an absent property) → None.
        _ => TextTransform::None,
    }
}

/// Applies `transform` to `content` **before** shaping (CSS Text 3 §2.1),
/// returning the cased string.
///
/// This is the pre-shape seam: the transformed string flows through itemization,
/// shaping, and the paragraph cache key, so a transformed node is a distinct
/// cache entry and the measure and compose passes — which both run `content`
/// through this same function — shape identical text.
///
/// [`TextTransform::None`] borrows `content` unchanged (zero-copy,
/// byte-identical). `uppercase`/`lowercase` use Rust's Unicode-aware
/// [`str::to_uppercase`]/[`str::to_lowercase`]. `capitalize` uppercases the first
/// **cased letter** of each whitespace-delimited word (the first letter, not a
/// leading symbol or digit), leaving the rest untouched. Locale-tailored casing
/// (e.g. Turkish dotless *i*) and the full grapheme/word-boundary subtleties are
/// documented refinements over this common Unicode path.
#[must_use]
pub(crate) fn transform_text(content: &str, transform: TextTransform) -> Cow<'_, str> {
    match transform {
        TextTransform::None => Cow::Borrowed(content),
        TextTransform::Uppercase => Cow::Owned(content.to_uppercase()),
        TextTransform::Lowercase => Cow::Owned(content.to_lowercase()),
        TextTransform::Capitalize => Cow::Owned(capitalize(content)),
    }
}

/// Uppercases the first cased letter of each whitespace-bounded word (§2.1
/// `capitalize`).
///
/// A word boundary is any Unicode whitespace run; within a word the first
/// character for which [`char::is_alphabetic`] holds is title/upper-cased and the
/// rest are copied verbatim. Leading punctuation or digits do not consume the
/// "first letter", matching the spec intent that `capitalize` targets the first
/// *letter* of the word.
fn capitalize(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    // `at_word_start` is set at the string start and after every whitespace run;
    // it is cleared once a word's first letter has been uppercased.
    let mut at_word_start = true;
    for ch in content.chars() {
        if ch.is_whitespace() {
            at_word_start = true;
            out.push(ch);
        } else if at_word_start && ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
            at_word_start = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::Length;

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

    #[test]
    fn alignment_defaults_to_start_and_maps_keywords() {
        // Absent property and an empty style both resolve to Start.
        assert_eq!(alignment_for(None), TextAlignment::Start);
        assert_eq!(
            alignment_for(Some(&ComputedStyle::new())),
            TextAlignment::Start
        );
        // Each reserved handle maps to its variant.
        let cases = [
            (layout_keyword::TEXT_ALIGN_LEFT, TextAlignment::Left),
            (layout_keyword::TEXT_ALIGN_RIGHT, TextAlignment::Right),
            (layout_keyword::TEXT_ALIGN_CENTER, TextAlignment::Center),
            (layout_keyword::TEXT_ALIGN_JUSTIFY, TextAlignment::Justify),
            (layout_keyword::TEXT_ALIGN_START, TextAlignment::Start),
            (layout_keyword::TEXT_ALIGN_END, TextAlignment::End),
        ];
        for (symbol, expected) in cases {
            let style = styled(PropertyId::TextAlign, PropertyValue::Keyword(symbol));
            assert_eq!(alignment_for(Some(&style)), expected);
        }
    }

    #[test]
    fn overflow_defaults_to_clip_and_maps_ellipsis() {
        // Absent property and an empty style both resolve to Clip (the CSS initial).
        assert_eq!(overflow_for(None), TextOverflow::Clip);
        assert_eq!(overflow_for(Some(&ComputedStyle::new())), TextOverflow::Clip);
        // Each reserved handle maps to its variant.
        let clip = styled(
            PropertyId::TextOverflow,
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_CLIP),
        );
        assert_eq!(overflow_for(Some(&clip)), TextOverflow::Clip);
        let ellipsis = styled(
            PropertyId::TextOverflow,
            PropertyValue::Keyword(layout_keyword::TEXT_OVERFLOW_ELLIPSIS),
        );
        assert_eq!(overflow_for(Some(&ellipsis)), TextOverflow::Ellipsis);
    }

    #[test]
    fn line_height_resolves_number_length_and_falls_back_on_normal() {
        // Absent property / empty style → None (caller uses the metric line height).
        assert_eq!(line_height_for(None, 16.0), None);
        assert_eq!(line_height_for(Some(&ComputedStyle::new()), 16.0), None);
        // A unitless number is a multiple of the font size: 1.5 × 16 = 24.
        let multiple = styled(PropertyId::LineHeight, PropertyValue::Number(1.5));
        assert_eq!(line_height_for(Some(&multiple), 16.0), Some(24.0));
        // The same multiple scales with the font size: 1.5 × 20 = 30.
        assert_eq!(line_height_for(Some(&multiple), 20.0), Some(30.0));
        // An absolute length is taken verbatim, independent of the font size.
        let length = styled(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::px(28.0)),
        );
        assert_eq!(line_height_for(Some(&length), 16.0), Some(28.0));
        // A keyword (`normal`) is unresolved here → None (metric fallback).
        let normal = styled(
            PropertyId::LineHeight,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL),
        );
        assert_eq!(line_height_for(Some(&normal), 16.0), None);
    }

    #[test]
    fn spacing_defaults_zero_and_reads_px_lengths() {
        // Absent property and empty style → 0 (no tracking / no word spacing).
        assert_eq!(letter_spacing_for(None), 0.0);
        assert_eq!(word_spacing_for(None), 0.0);
        assert_eq!(letter_spacing_for(Some(&ComputedStyle::new())), 0.0);
        // `normal` (a keyword) resolves to 0 for both.
        let ls_normal = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL),
        );
        assert_eq!(letter_spacing_for(Some(&ls_normal)), 0.0);
        // An absolute pixel length resolves verbatim, including a negative one.
        let ls = styled(PropertyId::LetterSpacing, PropertyValue::Length(Length::px(6.0)));
        assert_eq!(letter_spacing_for(Some(&ls)), 6.0);
        let ws = styled(PropertyId::WordSpacing, PropertyValue::Length(Length::px(12.0)));
        assert_eq!(word_spacing_for(Some(&ws)), 12.0);
        let neg = styled(PropertyId::LetterSpacing, PropertyValue::Length(Length::px(-2.0)));
        assert_eq!(letter_spacing_for(Some(&neg)), -2.0);
        // A percentage / non-pixel length is not resolved here → 0.
        let pct = styled(PropertyId::LetterSpacing, PropertyValue::Length(Length::Percent(50.0)));
        assert_eq!(letter_spacing_for(Some(&pct)), 0.0);
    }

    #[test]
    fn text_transform_defaults_none_and_maps_keywords() {
        assert_eq!(text_transform_for(None), TextTransform::None);
        assert_eq!(
            text_transform_for(Some(&ComputedStyle::new())),
            TextTransform::None
        );
        let cases = [
            (layout_keyword::TEXT_TRANSFORM_NONE, TextTransform::None),
            (layout_keyword::TEXT_TRANSFORM_UPPERCASE, TextTransform::Uppercase),
            (layout_keyword::TEXT_TRANSFORM_LOWERCASE, TextTransform::Lowercase),
            (layout_keyword::TEXT_TRANSFORM_CAPITALIZE, TextTransform::Capitalize),
        ];
        for (symbol, expected) in cases {
            let style = styled(PropertyId::TextTransform, PropertyValue::Keyword(symbol));
            assert_eq!(text_transform_for(Some(&style)), expected);
        }
    }

    #[test]
    fn transform_text_none_is_zero_copy_identity() {
        let out = transform_text("Hello World", TextTransform::None);
        assert!(matches!(out, Cow::Borrowed("Hello World")));
    }

    #[test]
    fn transform_text_cases_and_capitalizes() {
        assert_eq!(transform_text("hello", TextTransform::Uppercase), "HELLO");
        assert_eq!(transform_text("HeLLo", TextTransform::Lowercase), "hello");
        // Capitalize (task definition: word = whitespace-bounded run; uppercase
        // its first *cased letter*, so leading symbols are skipped over).
        assert_eq!(
            transform_text("hello world", TextTransform::Capitalize),
            "Hello World"
        );
        assert_eq!(
            transform_text("(quoted) words", TextTransform::Capitalize),
            "(Quoted) Words"
        );
        // Already-capital and non-letter-led words are left/handled sensibly.
        assert_eq!(
            transform_text("ÜBER caf\u{00e9}", TextTransform::Capitalize),
            "ÜBER Caf\u{00e9}"
        );
    }
}
