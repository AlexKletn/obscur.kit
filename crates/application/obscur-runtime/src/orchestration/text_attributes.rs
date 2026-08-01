#![doc = include_str!("text_attributes.doc/_module.md")]

use std::borrow::Cow;

use obscur_style::computed::ComputedStyle;
use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
use obscur_text::font::FontAttributes;
use obscur_text::layout::{OverflowWrap, TextAlignment, TextOverflow, WhiteSpace, WordBreak};
use obscur_text::shaping::TextDirection;
use obscur_text::span::{FontSlant, FontWeight};

use super::EditingView;

#[doc = include_str!("text_attributes.doc/attributes_for.md")]
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

#[doc = include_str!("text_attributes.doc/weight_from.md")]
fn weight_from(weight: f32) -> FontWeight {
    // The weight is rounded to the nearest whole step and clamped into the
    // `1..=1000` axis before the (already-clamping) `FontWeight::new`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = weight.round().clamp(0.0, 1000.0) as u16;
    FontWeight::new(value)
}

#[doc = include_str!("text_attributes.doc/slant_from.md")]
fn slant_from(symbol: obscur_identity::InternedSymbol) -> FontSlant {
    if symbol == layout_keyword::FONT_STYLE_ITALIC {
        FontSlant::Italic
    } else if symbol == layout_keyword::FONT_STYLE_OBLIQUE {
        FontSlant::Oblique
    } else {
        FontSlant::Normal
    }
}

#[doc = include_str!("text_attributes.doc/direction_for.md")]
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

#[doc = include_str!("text_attributes.doc/alignment_for.md")]
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

#[doc = include_str!("text_attributes.doc/overflow_for.md")]
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

#[doc = include_str!("text_attributes.doc/line_height_for.md")]
#[must_use]
pub(crate) fn line_height_for(
    computed: Option<&ComputedStyle>,
    font_size: f32,
    root_font_size: f32,
) -> Option<f32> {
    match computed?.get(PropertyId::LineHeight)?.value() {
        PropertyValue::Number(multiple) => Some(multiple * font_size),
        // CSS 2.1 §10.8.1: a percentage line-height is that percent of the
        // element's own computed `font-size`.
        PropertyValue::Length(Length::Percent(pct)) => Some(pct / 100.0 * font_size),
        PropertyValue::Length(length) => length_to_px(*length, font_size, root_font_size),
        _ => None,
    }
}

#[doc = include_str!("text_attributes.doc/length_to_px.md")]
fn length_to_px(length: Length, font_size: f32, root_font_size: f32) -> Option<f32> {
    match length {
        Length::Px(px) => Some(px.0),
        Length::Em(factor) => Some(factor * font_size),
        Length::Rem(factor) => Some(factor * root_font_size),
        _ => None,
    }
}

#[doc = include_str!("text_attributes.doc/letter_spacing_for.md")]
#[must_use]
pub(crate) fn letter_spacing_for(
    computed: Option<&ComputedStyle>,
    font_size: f32,
    root_font_size: f32,
) -> f32 {
    length_px_of(
        computed,
        PropertyId::LetterSpacing,
        font_size,
        root_font_size,
    )
}

#[doc = include_str!("text_attributes.doc/word_spacing_for.md")]
#[must_use]
pub(crate) fn word_spacing_for(
    computed: Option<&ComputedStyle>,
    font_size: f32,
    root_font_size: f32,
) -> f32 {
    length_px_of(computed, PropertyId::WordSpacing, font_size, root_font_size)
}

#[doc = include_str!("text_attributes.doc/length_px_of.md")]
fn length_px_of(
    computed: Option<&ComputedStyle>,
    property: PropertyId,
    font_size: f32,
    root_font_size: f32,
) -> f32 {
    let Some(value) = computed.and_then(|style| style.get(property)) else {
        return 0.0;
    };
    match value.value() {
        PropertyValue::Length(length) => {
            length_to_px(*length, font_size, root_font_size).unwrap_or(0.0)
        }
        _ => 0.0,
    }
}

#[doc = include_str!("text_attributes.doc/TextTransform.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextTransform {
    #[doc = include_str!("text_attributes.doc/None.md")]
    None,
    #[doc = include_str!("text_attributes.doc/Uppercase.md")]
    Uppercase,
    #[doc = include_str!("text_attributes.doc/Lowercase.md")]
    Lowercase,
    #[doc = include_str!("text_attributes.doc/Capitalize.md")]
    Capitalize,
}

#[doc = include_str!("text_attributes.doc/text_transform_for.md")]
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

#[doc = include_str!("text_attributes.doc/transform_text.md")]
#[must_use]
pub(crate) fn transform_text(content: &str, transform: TextTransform) -> Cow<'_, str> {
    match transform {
        TextTransform::None => Cow::Borrowed(content),
        TextTransform::Uppercase => Cow::Owned(content.to_uppercase()),
        TextTransform::Lowercase => Cow::Owned(content.to_lowercase()),
        TextTransform::Capitalize => Cow::Owned(capitalize(content)),
    }
}

#[doc = include_str!("text_attributes.doc/capitalize.md")]
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

#[doc = include_str!("text_attributes.doc/white_space_for.md")]
#[must_use]
pub(crate) fn white_space_for(computed: Option<&ComputedStyle>) -> WhiteSpace {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::WhiteSpace))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    match symbol {
        Some(s) if s == layout_keyword::WHITE_SPACE_NOWRAP => WhiteSpace::Nowrap,
        Some(s) if s == layout_keyword::WHITE_SPACE_PRE => WhiteSpace::Pre,
        Some(s) if s == layout_keyword::WHITE_SPACE_PRE_WRAP => WhiteSpace::PreWrap,
        Some(s) if s == layout_keyword::WHITE_SPACE_PRE_LINE => WhiteSpace::PreLine,
        // `normal` and everything else (including an absent property) → Normal.
        _ => WhiteSpace::Normal,
    }
}

#[doc = include_str!("text_attributes.doc/overflow_wrap_for.md")]
#[must_use]
pub(crate) fn overflow_wrap_for(computed: Option<&ComputedStyle>) -> OverflowWrap {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::OverflowWrap))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    match symbol {
        Some(s) if s == layout_keyword::OVERFLOW_WRAP_BREAK_WORD => OverflowWrap::BreakWord,
        Some(s) if s == layout_keyword::OVERFLOW_WRAP_ANYWHERE => OverflowWrap::Anywhere,
        // `normal` and everything else (including an absent property) → Normal.
        _ => OverflowWrap::Normal,
    }
}

#[doc = include_str!("text_attributes.doc/word_break_for.md")]
#[must_use]
pub(crate) fn word_break_for(computed: Option<&ComputedStyle>) -> WordBreak {
    let symbol = computed
        .and_then(|style| style.get(PropertyId::WordBreak))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        });
    match symbol {
        Some(s) if s == layout_keyword::WORD_BREAK_BREAK_ALL => WordBreak::BreakAll,
        Some(s) if s == layout_keyword::WORD_BREAK_KEEP_ALL => WordBreak::KeepAll,
        // `normal` and everything else (including an absent property) → Normal.
        _ => WordBreak::Normal,
    }
}

#[doc = include_str!("text_attributes.doc/collapse_white_space.md")]
#[must_use]
pub(crate) fn collapse_white_space(content: &str, white_space: WhiteSpace) -> Cow<'_, str> {
    if !white_space.collapses_spaces() {
        // `pre` / `pre-wrap`: preserve spaces, tabs, and segment breaks. Only the
        // CR-normalisation of segment breaks applies (§4.1.1 step 1).
        if content.contains('\r') {
            return Cow::Owned(content.replace("\r\n", "\n").replace('\r', "\n"));
        }
        return Cow::Borrowed(content);
    }
    // Collapsing modes (`normal` / `nowrap` / `pre-line`). Fast path: nothing to
    // change (no segment break, no tab, and no collapsible space run or edge).
    if !content.contains('\r')
        && !content.contains('\n')
        && !content.contains('\t')
        && !content.starts_with(' ')
        && !content.ends_with(' ')
        && !content.contains("  ")
    {
        return Cow::Borrowed(content);
    }
    let preserve_breaks = white_space.preserves_segment_breaks();
    let mut out = String::with_capacity(content.len());
    // `pending_space` marks a collapsible space that will be emitted before the
    // next non-space; `at_line_start` suppresses leading collapsible whitespace at
    // the block start and after each preserved forced break (edge trimming).
    let mut pending_space = false;
    let mut at_line_start = true;
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        // Normalise a CRLF / CR segment break to a single `\n`.
        let ch = if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            '\n'
        } else {
            ch
        };
        if ch == '\n' {
            if preserve_breaks {
                // A preserved forced break: trim the collapsible whitespace around
                // it (drop the pending space) and start a fresh line.
                pending_space = false;
                out.push('\n');
                at_line_start = true;
            } else if !at_line_start {
                // `normal` / `nowrap`: the segment break collapses to a space.
                pending_space = true;
            }
        } else if ch == ' ' || ch == '\t' {
            if !at_line_start {
                pending_space = true;
            }
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
            at_line_start = false;
        }
    }
    // A trailing pending space is the block's trailing whitespace: trimmed (never
    // emitted).
    Cow::Owned(out)
}

#[doc = include_str!("text_attributes.doc/process_text.md")]
#[must_use]
pub(crate) fn process_text(
    content: &str,
    transform: TextTransform,
    white_space: WhiteSpace,
) -> Cow<'_, str> {
    let transformed = transform_text(content, transform);
    // Collapse over the transformed text. If collapsing changes nothing, keep the
    // (possibly borrowed) transformed Cow so a pure-transform node stays zero-copy.
    match collapse_white_space(&transformed, white_space) {
        Cow::Borrowed(_) => transformed,
        Cow::Owned(collapsed) => Cow::Owned(collapsed),
    }
}

#[doc = include_str!("text_attributes.doc/display_text.md")]
#[must_use]
pub(crate) fn display_text<'a>(
    content: &'a str,
    active: Option<EditingView<'a>>,
    transform: TextTransform,
    white_space: WhiteSpace,
) -> Cow<'a, str> {
    let Some(view) = active else {
        // Idle (no session on this node): the ordinary pre-shape pipeline.
        return process_text(content, transform, white_space);
    };
    match view.preedit {
        // Composing: splice the uncommitted preedit into the buffer text for
        // DISPLAY only (spec §5.12) and shape the result RAW. The preedit
        // **replaces** the composition anchor span `[at, end)`, so composing over a
        // non-empty selection shows just the preedit where the selection was (the
        // selected text is hidden while composing) rather than the preedit beside
        // the still-selected text. With a collapsed anchor (`at == end`, the common
        // no-selection case) this is a pure insertion at `at` — byte-identical to
        // the previous splice. The buffer is untouched; only the display changes.
        Some(preedit) => {
            let at = preedit.at.min(view.text.len());
            let end = preedit.end.clamp(at, view.text.len());
            let mut spliced = String::with_capacity(view.text.len() + preedit.text.len());
            spliced.push_str(&view.text[..at]);
            spliced.push_str(preedit.text);
            spliced.push_str(&view.text[end..]);
            Cow::Owned(spliced)
        }
        // Editing without a live composition: shape the raw session buffer, so the
        // caret's buffer offsets equal the shaped-string offsets.
        None => Cow::Borrowed(view.text),
    }
}

#[doc = include_str!("text_attributes.doc/TabSize.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TabSize {
    #[doc = include_str!("text_attributes.doc/Factor.md")]
    Factor(f32),
    #[doc = include_str!("text_attributes.doc/Length.md")]
    Length(f32),
}

impl TabSize {
    #[doc = include_str!("text_attributes.doc/resolve.md")]
    #[must_use]
    pub(crate) fn resolve(self, space_advance: f32) -> f32 {
        match self {
            Self::Factor(factor) => factor * space_advance,
            Self::Length(px) => px,
        }
    }
}

#[doc = include_str!("text_attributes.doc/tab_size_for.md")]
#[must_use]
pub(crate) fn tab_size_for(
    computed: Option<&ComputedStyle>,
    font_size: f32,
    root_font_size: f32,
) -> TabSize {
    let Some(value) = computed.and_then(|style| style.get(PropertyId::TabSize)) else {
        return TabSize::Factor(8.0);
    };
    match value.value() {
        PropertyValue::Number(factor) => TabSize::Factor(factor.max(0.0)),
        PropertyValue::Length(length) => length_to_px(*length, font_size, root_font_size)
            .map_or(TabSize::Factor(8.0), |px| TabSize::Length(px.max(0.0))),
        _ => TabSize::Factor(8.0),
    }
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
        assert_eq!(
            overflow_for(Some(&ComputedStyle::new())),
            TextOverflow::Clip
        );
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
        assert_eq!(line_height_for(None, 16.0, 16.0), None);
        assert_eq!(
            line_height_for(Some(&ComputedStyle::new()), 16.0, 16.0),
            None
        );
        // A unitless number is a multiple of the font size: 1.5 × 16 = 24.
        let multiple = styled(PropertyId::LineHeight, PropertyValue::Number(1.5));
        assert_eq!(line_height_for(Some(&multiple), 16.0, 16.0), Some(24.0));
        // The same multiple scales with the font size: 1.5 × 20 = 30.
        assert_eq!(line_height_for(Some(&multiple), 20.0, 16.0), Some(30.0));
        // An absolute length is taken verbatim, independent of the font size
        // (byte-identical to the previous `as_px` path).
        let length = styled(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::px(28.0)),
        );
        assert_eq!(line_height_for(Some(&length), 16.0, 16.0), Some(28.0));
        // A keyword (`normal`) is unresolved here → None (metric fallback).
        let normal = styled(
            PropertyId::LineHeight,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL),
        );
        assert_eq!(line_height_for(Some(&normal), 16.0, 16.0), None);
    }

    #[test]
    fn line_height_resolves_em_rem_and_percent() {
        // `%` (CSS 2.1 §10.8.1): 150% × font-size 16 = 24.
        let pct = styled(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::Percent(150.0)),
        );
        assert_eq!(line_height_for(Some(&pct), 16.0, 16.0), Some(24.0));
        // `em` (CSS Values 4 §5.1.1): 2em × font-size 16 = 32.
        let em = styled(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::Em(2.0)),
        );
        assert_eq!(line_height_for(Some(&em), 16.0, 16.0), Some(32.0));
        // `rem` resolves against the ROOT font-size (10), NOT the node's (16):
        // 1.5rem × 10 = 15.
        let rem = styled(
            PropertyId::LineHeight,
            PropertyValue::Length(Length::Rem(1.5)),
        );
        assert_eq!(line_height_for(Some(&rem), 16.0, 10.0), Some(15.0));
    }

    #[test]
    fn spacing_defaults_zero_and_reads_px_lengths() {
        // Absent property and empty style → 0 (no tracking / no word spacing).
        assert_eq!(letter_spacing_for(None, 16.0, 16.0), 0.0);
        assert_eq!(word_spacing_for(None, 16.0, 16.0), 0.0);
        assert_eq!(
            letter_spacing_for(Some(&ComputedStyle::new()), 16.0, 16.0),
            0.0
        );
        // `normal` (a keyword) resolves to 0 for both.
        let ls_normal = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Keyword(layout_keyword::FONT_STYLE_NORMAL),
        );
        assert_eq!(letter_spacing_for(Some(&ls_normal), 16.0, 16.0), 0.0);
        // An absolute pixel length resolves verbatim, including a negative one
        // (byte-identical to the previous `as_px` path).
        let ls = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Length(Length::px(6.0)),
        );
        assert_eq!(letter_spacing_for(Some(&ls), 16.0, 16.0), 6.0);
        let ws = styled(
            PropertyId::WordSpacing,
            PropertyValue::Length(Length::px(12.0)),
        );
        assert_eq!(word_spacing_for(Some(&ws), 16.0, 16.0), 12.0);
        let neg = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Length(Length::px(-2.0)),
        );
        assert_eq!(letter_spacing_for(Some(&neg), 16.0, 16.0), -2.0);
        // A percentage is INVALID for letter-/word-spacing (CSS Text 3 §8.1/§8.2)
        // and is not resolved here → 0.
        let pct = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Length(Length::Percent(50.0)),
        );
        assert_eq!(letter_spacing_for(Some(&pct), 16.0, 16.0), 0.0);
    }

    #[test]
    fn spacing_resolves_em_against_font_size_and_rem_against_root() {
        // `letter-spacing: 0.25em` at font-size 16 → 0.25 × 16 = 4.
        let ls_em = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Length(Length::Em(0.25)),
        );
        assert_eq!(letter_spacing_for(Some(&ls_em), 16.0, 16.0), 4.0);
        // `word-spacing: 0.25em` at font-size 16 → 4 (resolved, not dropped to 0).
        let ws_em = styled(
            PropertyId::WordSpacing,
            PropertyValue::Length(Length::Em(0.25)),
        );
        assert_eq!(word_spacing_for(Some(&ws_em), 16.0, 16.0), 4.0);
        // `letter-spacing: 1rem` resolves against the ROOT font-size (20), not the
        // node's (16): 1 × 20 = 20.
        let ls_rem = styled(
            PropertyId::LetterSpacing,
            PropertyValue::Length(Length::Rem(1.0)),
        );
        assert_eq!(letter_spacing_for(Some(&ls_rem), 16.0, 20.0), 20.0);
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
            (
                layout_keyword::TEXT_TRANSFORM_UPPERCASE,
                TextTransform::Uppercase,
            ),
            (
                layout_keyword::TEXT_TRANSFORM_LOWERCASE,
                TextTransform::Lowercase,
            ),
            (
                layout_keyword::TEXT_TRANSFORM_CAPITALIZE,
                TextTransform::Capitalize,
            ),
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
    fn white_space_defaults_normal_and_maps_keywords() {
        assert_eq!(white_space_for(None), WhiteSpace::Normal);
        assert_eq!(
            white_space_for(Some(&ComputedStyle::new())),
            WhiteSpace::Normal
        );
        let cases = [
            (layout_keyword::WHITE_SPACE_NORMAL, WhiteSpace::Normal),
            (layout_keyword::WHITE_SPACE_NOWRAP, WhiteSpace::Nowrap),
            (layout_keyword::WHITE_SPACE_PRE, WhiteSpace::Pre),
            (layout_keyword::WHITE_SPACE_PRE_WRAP, WhiteSpace::PreWrap),
            (layout_keyword::WHITE_SPACE_PRE_LINE, WhiteSpace::PreLine),
        ];
        for (symbol, expected) in cases {
            let style = styled(PropertyId::WhiteSpace, PropertyValue::Keyword(symbol));
            assert_eq!(white_space_for(Some(&style)), expected);
        }
    }

    #[test]
    fn collapse_single_line_no_run_is_zero_copy_identity() {
        // The byte-identical guard: single-spaced text with no `\n`/`\t` and no
        // leading/trailing space collapses to itself, zero-copy.
        let out = collapse_white_space("a b c", WhiteSpace::Normal);
        assert!(matches!(out, Cow::Borrowed("a b c")));
    }

    #[test]
    fn collapse_normal_folds_runs_and_newlines_to_single_space() {
        // "a    b" (4 spaces) → "a b"; a `\n` collapses to a space (§4.1.1).
        assert_eq!(collapse_white_space("a    b", WhiteSpace::Normal), "a b");
        assert_eq!(collapse_white_space("a\nb", WhiteSpace::Normal), "a b");
        // Leading/trailing whitespace of the block is trimmed.
        assert_eq!(collapse_white_space("  a  b  ", WhiteSpace::Normal), "a b");
        // A tab is collapsible in `normal` and folds into the space run.
        assert_eq!(collapse_white_space("a\t\tb", WhiteSpace::Normal), "a b");
    }

    #[test]
    fn collapse_pre_preserves_spaces_and_newlines() {
        // `pre` preserves runs of spaces, tabs, and segment breaks verbatim.
        let out = collapse_white_space("a    b\n\tc", WhiteSpace::Pre);
        assert_eq!(out, "a    b\n\tc");
        assert!(
            matches!(out, Cow::Borrowed(_)),
            "pre with no CR is zero-copy"
        );
        // `\r\n` / `\r` normalise to a single `\n` even under a preserving mode.
        assert_eq!(
            collapse_white_space("a\r\nb\rc", WhiteSpace::Pre),
            "a\nb\nc"
        );
    }

    #[test]
    fn collapse_pre_line_collapses_spaces_but_preserves_breaks() {
        // `pre-line`: runs of spaces collapse to one, but `\n` is preserved, and
        // whitespace around the forced break is trimmed.
        assert_eq!(
            collapse_white_space("a    b \n  c", WhiteSpace::PreLine),
            "a b\nc"
        );
    }

    #[test]
    fn tab_size_defaults_eight_and_reads_number_and_length() {
        // Absent property / empty style → the initial `8` spaces.
        assert_eq!(tab_size_for(None, 16.0, 16.0), TabSize::Factor(8.0));
        assert_eq!(
            tab_size_for(Some(&ComputedStyle::new()), 16.0, 16.0),
            TabSize::Factor(8.0)
        );
        // A unitless number is a multiple of the space advance.
        let number = styled(PropertyId::TabSize, PropertyValue::Number(4.0));
        assert_eq!(
            tab_size_for(Some(&number), 16.0, 16.0),
            TabSize::Factor(4.0)
        );
        assert_eq!(TabSize::Factor(4.0).resolve(10.0), 40.0);
        // An absolute length is a fixed tab-stop width (byte-identical to the
        // previous `as_px` path).
        let length = styled(PropertyId::TabSize, PropertyValue::Length(Length::px(24.0)));
        assert_eq!(
            tab_size_for(Some(&length), 16.0, 16.0),
            TabSize::Length(24.0)
        );
        assert_eq!(TabSize::Length(24.0).resolve(10.0), 24.0);
    }

    #[test]
    fn tab_size_resolves_em_and_rem_lengths() {
        // `tab-size: 2em` at font-size 16 → a fixed 2 × 16 = 32px tab stop.
        let em = styled(PropertyId::TabSize, PropertyValue::Length(Length::Em(2.0)));
        assert_eq!(tab_size_for(Some(&em), 16.0, 16.0), TabSize::Length(32.0));
        // `tab-size: 1rem` resolves against the ROOT font-size (20): 1 × 20 = 20px.
        let rem = styled(PropertyId::TabSize, PropertyValue::Length(Length::Rem(1.0)));
        assert_eq!(tab_size_for(Some(&rem), 16.0, 20.0), TabSize::Length(20.0));
        // A percentage is invalid for `tab-size` → the initial `8` spaces.
        let pct = styled(
            PropertyId::TabSize,
            PropertyValue::Length(Length::Percent(50.0)),
        );
        assert_eq!(tab_size_for(Some(&pct), 16.0, 16.0), TabSize::Factor(8.0));
    }

    #[test]
    fn process_text_composes_transform_then_collapse() {
        // uppercase then collapse the doubled spaces (spec order: cases first).
        assert_eq!(
            process_text("a    b", TextTransform::Uppercase, WhiteSpace::Normal),
            "A B"
        );
        // A pure no-op stays borrowed (byte-identical).
        let out = process_text("a b", TextTransform::None, WhiteSpace::Normal);
        assert!(matches!(out, Cow::Borrowed("a b")));
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
